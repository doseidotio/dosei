use crate::account::Account;
use crate::certificate::Certificate;
use crate::deployment::Deployment;
use crate::ingress::Ingress;
use crate::service::Service;
use crate::session::AuthSession;
use axum::extract::{Multipart, Path};
use axum::http::StatusCode;
use axum::{Extension, Json};
use dosei_schema::app::App;
use dosei_schema::cluster::ClusterInit;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::error;
use utoipa::gen::serde_json::{json, Value};
use uuid::Uuid;

const TAG: &str = "deployment";

#[utoipa::path(
  get,
  path = "/service/{service_id}/deployment",
  params(
    ("service_id" = String, Path, description = "Service ID"),
  ),
  responses(
        (status = StatusCode::OK, body = Vec<Deployment>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_service_deployments(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
  Path(service_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Deployment>>), StatusCode> {
  let service = Service::get_by_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
  if service.owner_id != session.account_id {
    return Err(StatusCode::NOT_FOUND);
  }
  let deployments = Deployment::get_by_service_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(deployments)))
}

#[utoipa::path(
  post,
  path = "/deploy",
  responses(
        (status = StatusCode::OK, body = Value),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_deploy(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
  mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
  let mut app = String::new();
  let mut hash = String::new();
  let mut file_data = Vec::new();
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?
  {
    if let Some(name) = field.name() {
      match name {
        "app" => {
          app = String::from_utf8(
            field
              .bytes()
              .await
              .map_err(|_| StatusCode::BAD_REQUEST)?
              .to_vec(),
          )
          .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
        "file" => {
          file_data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .to_vec();
        }
        "hash" => {
          hash = String::from_utf8(
            field
              .bytes()
              .await
              .map_err(|_| StatusCode::BAD_REQUEST)?
              .to_vec(),
          )
          .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
        _ => {} // Ignore other fields
      }
    }
  }

  let app = App::from_string(&app).map_err(|_| StatusCode::BAD_REQUEST)?;

  let service = match Service::new(&app.name, session.account_id, &pg_pool).await {
    Ok(service) => service,
    Err(_) => Service::get_by_name(app.name.clone(), &pg_pool)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
      .unwrap(),
  };

  let deployment = Deployment::new(service.id, service.owner_id, app.port, None, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  deployment
    .build(&file_data)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  deployment
    .start(None)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  if let Some(domains) = app.domains {
    if !domains.is_empty() {
      let domain = domains.first().unwrap();
      if let Ok(result) = Certificate::get_by_domain_name(domain.clone(), &pg_pool).await {
        if result.is_none() && ClusterInit::validate_domain(domain) {
          let account = Account::get_by_id(service.owner_id, &pg_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
          if let Err(e) = Certificate::request(account.unwrap().id, domain).await {
            error!("{}", e);
          }
          {
            let _ = Ingress::new(domain.clone(), service.id, service.owner_id, &pg_pool).await;
          }
        }
      }
    }
  }
  Ok((StatusCode::OK, Json(json!({}))))
}
