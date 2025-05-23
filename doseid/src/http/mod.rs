mod health;
mod info;
mod proxy;

use crate::config::Config;
use crate::http::proxy::Proxy;
use crate::session::Session;
use crate::{account, auth, certificate, deployment, ingress, service};
use anyhow::{anyhow, Context};
use axum::{middleware, Extension, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::Components;
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
  modifiers(&SecurityAddon)
)]
struct ApiDoc;

pub struct Http;

impl Http {
  pub async fn start_server(
    config: &'static Config,
    shared_pool: &Arc<Pool<Postgres>>,
  ) -> anyhow::Result<()> {
    let mut api_doc = ApiDoc::openapi();

    let (public_router, public_api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
      .routes(routes!(health::health))
      .routes(routes!(info::info))
      .routes(routes!(certificate::route::api_http01_challenge))
      .split_for_parts();
    api_doc.merge(public_api);

    let (private_router, private_api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
      .routes(routes!(account::route::api_user))
      .routes(routes!(account::route::api_list_user_ssh_key))
      .routes(routes!(certificate::route::api_list_certificates))
      .routes(routes!(service::route::api_list_services))
      .routes(routes!(deployment::route::api_deploy))
      .routes(routes!(deployment::route::api_list_service_deployments))
      .routes(routes!(ingress::route::api_list_service_ingresses))
      .routes(routes!(auth::route::login_ssh))
      .routes(routes!(auth::route::logout))
      .route_layer(middleware::from_fn(Session::middleware))
      .split_for_parts();
    api_doc.merge(private_api);

    let app = Router::new()
      .merge(public_router)
      .merge(private_router)
      .merge(SwaggerUi::new("/docs").url("/openapi.json", api_doc))
      .layer(CorsLayer::permissive())
      .layer(Extension(Arc::clone(shared_pool)))
      .layer(Extension(config));

    let listener = TcpListener::bind(&config.address())
      .await
      .context("Failed to start server")?;
    if let Err(e) = Proxy::start_server(config, shared_pool).await {
      error!("Failed to start proxy server: {}", e);
    }
    tokio::spawn(async move {
      info!(
        "DoseidD API running on http://{} (Press Ctrl+C to quit)",
        &config.address()
      );
      axum::serve(listener, app)
        .await
        .expect("Failed start DoseiD API");
    });
    signal::ctrl_c()
      .await
      .map_err(|err| anyhow!("Unable to listen for shutdown signal: {}", err))?;
    info!("Gracefully stopping... (Press Ctrl+C again to force)");
    Ok(())
  }
}

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if openapi.components.is_none() {
      openapi.components = Some(Components::new());
    }

    openapi.components.as_mut().unwrap().add_security_scheme(
      "Authentication",
      SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
    );
  }
}
