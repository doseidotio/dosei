use crate::session::{AuthSession, Session, SessionCredentials};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

const TAG: &str = "auth";

#[utoipa::path(
    post,
    path = "/auth/login/ssh",
    responses(
        (status = StatusCode::OK, description = "Log in successfully", body = SessionCredentials),
    ),
    tag = TAG,
)]
pub async fn login_ssh(
  pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<(StatusCode, Json<SessionCredentials>), StatusCode> {
  let session = Session::new(session.account_id, &pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::CREATED, Json(session.to_credentials())))
}

#[utoipa::path(
    delete,
    path = "/auth/logout",
    responses(
        (status = StatusCode::OK, description = "Logged out successfully"),
    ),
    security(
      ("Authentication" = [])
    ),
    tag = TAG
)]
pub async fn logout(
  pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<StatusCode, StatusCode> {
  Session::delete(session.token, &pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(StatusCode::OK)
}
