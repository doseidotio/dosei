use crate::ingress::Ingress;
use crate::service::Service;
use crate::session::AuthSession;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

const TAG: &str = "ingress";

#[utoipa::path(
  get,
  path = "/service/{service_id}/ingress",
  params(
    ("service_id" = String, Path, description = "Service ID"),
  ),
  responses(
        (status = StatusCode::OK, body = Vec<Ingress>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_service_ingresses(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
  Path(service_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Ingress>>), StatusCode> {
  let service = Service::get_by_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
  if service.owner_id != session.account_id {
    return Err(StatusCode::NOT_FOUND);
  }
  let ingresses = Ingress::get_by_service_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(ingresses)))
}
