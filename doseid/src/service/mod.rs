use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod route;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Service {
  pub id: Uuid,
  pub name: String,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Service {
  pub async fn new(
    name: &str,
    owner_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Service> {
    let service = sqlx::query_as!(
      Service,
      "INSERT INTO service (id, name, owner_id, updated_at, created_at)
       VALUES ($1, $2, $3, $4, $5)
       RETURNING *
      ",
      Uuid::new_v4(),
      name,
      owner_id,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!("Created service: {}", service.name);
    Ok(service)
  }

  pub async fn get_by_id(id: Uuid, pg_pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
    Ok(
      sqlx::query_as!(Service, "SELECT * FROM service WHERE id = $1", id)
        .fetch_optional(pg_pool)
        .await?,
    )
  }

  pub async fn get_by_name(name: String, pg_pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
    Ok(
      sqlx::query_as!(Service, "SELECT * FROM service WHERE name = $1", name)
        .fetch_optional(pg_pool)
        .await?,
    )
  }

  pub async fn get_by_owner_id(
    owner_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(
        Service,
        "SELECT * FROM service WHERE owner_id = $1",
        owner_id
      )
      .fetch_all(pg_pool)
      .await?,
    )
  }
}
