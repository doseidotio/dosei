use crate::service::Service;
use crate::session::AuthSession;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

const TAG: &str = "service";

#[utoipa::path(
  get,
  path = "/service",
  responses(
        (status = StatusCode::OK, body = Vec<Service>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_services(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<(StatusCode, Json<Vec<Service>>), StatusCode> {
  let services = Service::get_by_owner_id(session.account_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(services)))
}
