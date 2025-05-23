use crate::account::{Account, AccountSSHKey};
use crate::session::AuthSession;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

const TAG: &str = "account";

#[utoipa::path(
  get,
  path = "/user",
  responses(
        (status = StatusCode::OK, body = Account),
        (status = StatusCode::NOT_FOUND, description = "Account Not Found"),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_user(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<(StatusCode, Json<Account>), StatusCode> {
  let account = Account::get_by_id(session.account_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
  Ok((StatusCode::OK, Json(account)))
}

#[utoipa::path(
  get,
  path = "/user/ssh-key",
  responses(
        (status = StatusCode::OK, body = Vec<AccountSSHKey>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_user_ssh_key(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
) -> Result<(StatusCode, Json<Vec<AccountSSHKey>>), StatusCode> {
  let account_ssh_keys = AccountSSHKey::get_by_owner_id(session.account_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(account_ssh_keys)))
}
