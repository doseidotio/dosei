use axum::http::StatusCode;

#[utoipa::path(
  get,
  path = "/health",
  responses(
            (status = OK, description = "Service is healthy"),
  ),
)]
pub async fn health() -> Result<StatusCode, StatusCode> {
  Ok(StatusCode::OK)
}
