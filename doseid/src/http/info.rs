use crate::cluster::Cluster;
use axum::http::StatusCode;
use axum::Json;
use utoipa::gen::serde_json::{json, Value};

#[utoipa::path(
  get,
  path = "/info",
  responses(
        (status = StatusCode::OK, body = Value),
  ),
)]
pub async fn info() -> Result<(StatusCode, Json<Value>), StatusCode> {
  Ok((
    StatusCode::OK,
    Json(json!({
        "name": Cluster::get().await.name,
        "version": env!("CARGO_PKG_VERSION"),
    })),
  ))
}
