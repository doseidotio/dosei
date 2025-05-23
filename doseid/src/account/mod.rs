use chrono::{DateTime, Utc};
use dosei_schema::ssh::SSHBearerPayload;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::info;
use uuid::Uuid;

pub mod route;

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct Account {
  pub id: Uuid,
  pub name: String,
  pub password: Option<String>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Account {
  pub async fn new(
    name: &str,
    password: Option<&str>,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Account> {
    let account = sqlx::query_as!(
      Account,
      "INSERT INTO account (id, name, password, updated_at, created_at)
       VALUES ($1, $2, $3, $4, $5)
       RETURNING *
      ",
      Uuid::new_v4(),
      name,
      password,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!("Created account: {}", account.name);
    Ok(account)
  }

  pub async fn delete(&self, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM account_ssh_key WHERE account_id = $1", self.id)
      .execute(pg_pool)
      .await?;
    sqlx::query!("DELETE FROM account WHERE id = $1", self.id)
      .execute(pg_pool)
      .await?;
    Ok(())
  }

  pub async fn get_all(pg_pool: &Pool<Postgres>) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(Account, "SELECT * FROM account")
        .fetch_all(pg_pool)
        .await?,
    )
  }

  pub async fn get_by_id(id: Uuid, pg_pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
    Ok(
      sqlx::query_as!(Account, "SELECT * FROM account WHERE id = $1", id,)
        .fetch_optional(pg_pool)
        .await?,
    )
  }

  pub async fn get_by_name(name: String, pg_pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
    Ok(
      sqlx::query_as!(Account, "SELECT * FROM account WHERE name = $1", name,)
        .fetch_optional(pg_pool)
        .await?,
    )
  }
  pub async fn get_default_user(pg_pool: &Pool<Postgres>) -> anyhow::Result<Self> {
    Ok(
      Account::get_by_name("dosei".to_string(), pg_pool)
        .await?
        .unwrap(),
    )
  }
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct AccountSSHKey {
  pub id: Uuid,
  pub key_fingerprint: String,
  pub ssh_key: String,
  pub account_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl AccountSSHKey {
  pub async fn new(
    account_id: Uuid,
    ssh_key: String,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Self> {
    let key_fingerprint = SSHBearerPayload::fingerprint_from_public_key(&ssh_key)?;
    let account_ssh_key = sqlx::query_as!(
      Self,
      "INSERT INTO account_ssh_key (id, key_fingerprint, ssh_key, account_id, updated_at, created_at)
       VALUES ($1, $2, $3, $4, $5, $6)
       RETURNING *
      ",
      Uuid::new_v4(),
      key_fingerprint,
      ssh_key,
      account_id,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!("Created SSH Key: {} for {}", ssh_key, account_id);
    Ok(account_ssh_key)
  }
  pub async fn get_by_fingerprint(
    key_fingerprint: String,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Self> {
    Ok(
      sqlx::query_as!(
        Self,
        "SELECT * FROM account_ssh_key WHERE key_fingerprint = $1",
        key_fingerprint
      )
      .fetch_one(pg_pool)
      .await?,
    )
  }

  pub async fn get_by_owner_id(
    owner_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(
        Self,
        "SELECT * FROM account_ssh_key WHERE account_id = $1",
        owner_id
      )
      .fetch_all(pg_pool)
      .await?,
    )
  }
}
