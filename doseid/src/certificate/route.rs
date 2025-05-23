use crate::certificate::Certificate;
use crate::certificate::CertificateManager;
use crate::session::AuthSession;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use log::info;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

const TAG: &str = "certificate";

#[utoipa::path(
  get,
  path = "/certificate",
  responses(
        (status = StatusCode::OK, body = Vec<Certificate>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_certificates(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<(StatusCode, Json<Vec<Certificate>>), StatusCode> {
  let certificates = Certificate::get_by_owner_id(session.account_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(certificates)))
}

#[utoipa::path(
  get,
  path = "/.well-known/acme-challenge/:token",
  responses(
        (status = StatusCode::OK, body = String, description = "Return the http01 challenge token value"),
        (status = StatusCode::NOT_FOUND, description = "Http01 challenge token not found"),
  ),
  tag = TAG
)]
pub async fn api_http01_challenge(
  Path(token): Path<String>,
) -> Result<(StatusCode, String), StatusCode> {
  info!("ACME challenge request for token: {}", token);
  if let Some(token_value) = CertificateManager::get_http01_challenge_token_value(token).await {
    return Ok((StatusCode::OK, token_value));
  }
  Err(StatusCode::NOT_FOUND)
}
