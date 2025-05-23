use crate::account::Account;
use crate::certificate::Certificate;
use crate::deployment::Deployment;
use crate::ingress::Ingress;
use crate::service::Service;
use dosei_schema::cluster::ClusterInit;
use sqlx::{Pool, Postgres};
use tracing::error;

pub struct Dashboard {
  // The domain name where the dashboard will be running.
  // dashboard.dosei.cloud
  pub name: String,
}

impl Dashboard {
  pub async fn init(&self, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let default_user = Account::get_default_user(pg_pool).await?;

    // Request a certificate for the domain name.
    if let Ok(result) = Certificate::get_by_domain_name(self.name.clone(), pg_pool).await {
      if result.is_none() && ClusterInit::validate_domain(&self.name) {
        if let Err(e) = Certificate::request(default_user.id, &self.name).await {
          error!("{}", e);
        }
      }
    }

    // Create dashboard Service if not present (aka: fresh cluster)
    let service = match Service::new("dashboard", default_user.id, pg_pool).await {
      Ok(service) => service,
      Err(_) => Service::get_by_name("dashboard".to_string(), pg_pool)
        .await?
        .unwrap(),
    };

    // Get or create a deployment for the service.
    let deployment = match Deployment::get_by_service_id(service.id, pg_pool)
      .await?
      .into_iter()
      .next()
    {
      Some(deployment) => deployment,
      None => {
        Deployment::new(
          service.id,
          service.owner_id,
          Some(8844),
          Some(8844),
          pg_pool,
        )
        .await?
      }
    };

    let image_tag = format!("doseidotio/dashboard:{}", env!("CARGO_PKG_VERSION"));
    deployment.stop().await?;
    deployment.remove().await?;
    deployment.start(Some(image_tag)).await?;

    // Ingress insert or Update
    match Ingress::get_by_service_id(service.id, pg_pool)
      .await?
      .first()
    {
      Some(ingress) => ingress.update_host(self.name.clone(), pg_pool).await?,
      None => Ingress::new(self.name.clone(), service.id, service.owner_id, pg_pool).await?,
    };
    Ok(())
  }
}
