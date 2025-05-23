use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod route;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Ingress {
  pub id: Uuid,
  pub service_id: Uuid,
  pub owner_id: Uuid,
  pub host: String,
  pub path: Option<String>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Ingress {
  pub async fn new(
    host: String,
    service_id: Uuid,
    owner_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Self> {
    let ingress = sqlx::query_as!(
      Self,
      "INSERT INTO
        ingress (
          id,
          service_id,
          owner_id,
          host,
          updated_at,
          created_at
        )
       VALUES ($1, $2, $3, $4, $5, $6)
       RETURNING *
      ",
      Uuid::new_v4(),
      service_id,
      owner_id,
      host,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!(
      "Created ingress: {} with host {:?}",
      ingress.id, ingress.host
    );
    Ok(ingress)
  }

  pub async fn get_by_service_id(
    service_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(
        Self,
        "SELECT * FROM ingress WHERE service_id = $1",
        service_id
      )
      .fetch_all(pg_pool)
      .await?,
    )
  }
  pub async fn update_host(&self, host: String, pg_pool: &Pool<Postgres>) -> anyhow::Result<Self> {
    Ok(
      sqlx::query_as!(
        Self,
        "UPDATE ingress SET host = $1, updated_at = $2 WHERE service_id = $3 RETURNING *",
        host,
        Utc::now(),
        self.service_id
      )
      .fetch_one(pg_pool)
      .await?,
    )
  }
}
