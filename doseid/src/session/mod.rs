use crate::account::AccountSSHKey;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use cached::{Cached, TimedCache};
use chrono::{DateTime, Utc};
use dosei_schema::ssh::SSHBearerPayload;
use once_cell::sync::Lazy;
use rand::distr::Alphanumeric;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

const BEARER: &str = "Bearer ";
const SSH: &str = "ssh:";

#[derive(Clone)]
pub struct AuthSession(pub Session);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
  pub account_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SessionCredentials {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
}

impl Session {
  pub fn to_credentials(&self) -> SessionCredentials {
    SessionCredentials {
      id: self.id,
      token: self.token.clone(),
      refresh_token: self.refresh_token.clone(),
    }
  }
}

static SESSION_CACHE: Lazy<Arc<Mutex<TimedCache<String, Session>>>> = Lazy::new(|| {
  let cache = TimedCache::with_lifespan(3600);
  Arc::new(Mutex::new(cache))
});

impl Session {
  pub async fn new(account_id: Uuid, pg_pool: &Pool<Postgres>) -> anyhow::Result<Self> {
    let session = sqlx::query_as!(
      Session,
      "
      INSERT INTO session (id, token, refresh_token, account_id, updated_at, created_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING *
      ",
      Uuid::new_v4(),
      rng()
        .sample_iter(&Alphanumeric)
        .take(96)
        .map(char::from)
        .collect::<String>(),
      rng()
        .sample_iter(&Alphanumeric)
        .take(96)
        .map(char::from)
        .collect::<String>(),
      account_id,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    let token = session.token.clone();
    let session_cloned = session.clone();
    {
      let session_cache = Arc::clone(&SESSION_CACHE);
      let mut cache = session_cache.lock().await;
      cache.cache_set(token, session_cloned);
    }
    Ok(session)
  }

  pub fn ssh_new(account_id: Uuid) -> Self {
    Self {
      id: Uuid::new_v4(),
      token: rng()
        .sample_iter(&Alphanumeric)
        .take(96)
        .map(char::from)
        .collect::<String>(),
      refresh_token: rng()
        .sample_iter(&Alphanumeric)
        .take(96)
        .map(char::from)
        .collect::<String>(),
      account_id,
      updated_at: Utc::now(),
      created_at: Utc::now(),
    }
  }

  pub async fn delete(token: String, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let session_cache = Arc::clone(&SESSION_CACHE);
    {
      let mut cache = session_cache.lock().await;
      cache.cache_remove(&token);
    }
    sqlx::query!("DELETE FROM session WHERE token = $1", token)
      .execute(pg_pool)
      .await?;
    Ok(())
  }

  async fn get_from_token(token: String, pg_pool: &Pool<Postgres>) -> Option<Session> {
    let session_cache = Arc::clone(&SESSION_CACHE);
    {
      let mut cache = session_cache.lock().await;
      if let Some(session) = cache.cache_get(&token).cloned() {
        cache.cache_set(token, session.clone());
        return Some(session);
      }
    }
    sqlx::query_as!(Session, "SELECT * FROM session WHERE token = $1", token,)
      .fetch_optional(pg_pool)
      .await
      .ok()?
  }

  pub async fn middleware(
    pool: Extension<Arc<Pool<Postgres>>>,
    mut request: Request,
    next: Next,
  ) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let authorization_header = headers
      .get(header::AUTHORIZATION)
      .ok_or(StatusCode::UNAUTHORIZED)?;
    let authorization = authorization_header
      .to_str()
      .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !authorization.starts_with(BEARER) {
      return Err(StatusCode::UNAUTHORIZED);
    }
    let bearer_token = authorization.trim_start_matches(BEARER);

    if bearer_token.starts_with(SSH) {
      let ssh_base64 = bearer_token.trim_start_matches(SSH);
      let payload =
        SSHBearerPayload::from_base64(ssh_base64).map_err(|_| StatusCode::UNAUTHORIZED)?;
      let key = AccountSSHKey::get_by_fingerprint(payload.key_fingerprint.clone(), &pool)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
      if !payload.verify_from_string(key.ssh_key) {
        return Err(StatusCode::UNAUTHORIZED);
      };
      let session = Session::ssh_new(key.account_id);
      request.extensions_mut().insert(AuthSession(session));
      Ok(next.run(request).await)
    } else {
      let session = Session::get_from_token(bearer_token.to_string(), &pool)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

      request.extensions_mut().insert(AuthSession(session));

      Ok(next.run(request).await)
    }
  }
}
