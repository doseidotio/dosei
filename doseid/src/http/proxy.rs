use crate::certificate::Certificate;
use crate::config::Config;
use crate::deployment::Deployment;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use rustls::pki_types::CertificateDer;
use rustls::server::ClientHello;
use rustls::sign::CertifiedKey;
use rustls::ServerConfig;
use sqlx::{Pool, Postgres};
use std::io::BufReader;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct Proxy;

impl Proxy {
  pub async fn start_server(
    config: &'static Config,
    shared_pool: &Arc<Pool<Postgres>>,
  ) -> anyhow::Result<()> {
    let connector = HttpConnector::new();
    let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(connector);

    let cert_resolver = DatabaseCertResolver::new(Arc::clone(shared_pool));

    let server_config = ServerConfig::builder()
      .with_no_client_auth()
      .with_cert_resolver(Arc::new(cert_resolver));

    let tls_config = RustlsConfig::from_config(Arc::new(server_config));

    let app = Router::new()
      .route("/", any(Self::handler))
      .route("/*path", any(Self::handler))
      .with_state(client)
      .layer(Extension(Arc::clone(shared_pool)))
      .layer(Extension(config));

    let listener = std::net::TcpListener::bind(config.proxy_address())?;
    tokio::spawn(async move {
      info!(
        "DoseiD Proxy Server running on http://{}",
        &config.proxy_address()
      );
      axum_server::from_tcp_rustls(listener, tls_config)
        .serve(app.into_make_service())
        .await
        .expect("Failed start Doseid Proxy Server");
    });
    Ok(())
  }

  async fn handler(
    pg_pool: Extension<Arc<Pool<Postgres>>>,
    State(client): State<Client>,
    mut req: Request,
  ) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let host = match headers.get("host") {
      Some(host_header) => host_header.to_str().unwrap_or_default(),
      None => return Err(StatusCode::NOT_FOUND),
    };
    debug!("Received request for host: {}", host);
    let path = req.uri().path();
    let path_query = req
      .uri()
      .path_and_query()
      .map(|v| v.as_str())
      .unwrap_or(path);

    match Deployment::find_via_host(host, &pg_pool)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
      None => Err(StatusCode::NOT_FOUND),
      Some(deployment) => match deployment.host_port {
        None => Err(StatusCode::NOT_FOUND),
        Some(host_port) => {
          let target_service = format!("http://127.0.0.1:{}{}", host_port, path_query);
          info!("Forwarding: {} -> {}", host, target_service);
          *req.uri_mut() = Uri::try_from(target_service).unwrap();
          let response = client
            .request(req)
            .await
            .map_err(|e| {
              error!("Request failed: {}", e);
              StatusCode::BAD_REQUEST
            })?
            .into_response();
          deployment.update_last_accessed(&pg_pool);
          Ok(response)
        }
      },
    }
  }
}

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[derive(Debug)]
struct DatabaseCertResolver {
  pool: Arc<Pool<Postgres>>,
}

impl DatabaseCertResolver {
  fn new(pool: Arc<Pool<Postgres>>) -> Self {
    Self { pool }
  }
}

impl rustls::server::ResolvesServerCert for DatabaseCertResolver {
  fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
    let server_name = client_hello.server_name()?;
    let domain = server_name.to_string();

    info!("Loading certificate for: {}", &domain);

    let pool = self.pool.clone();
    let domain_clone = domain.clone();
    let db_cert = tokio::task::block_in_place(move || {
      let rt = tokio::runtime::Handle::current();
      rt.block_on(async { Certificate::get_by_domain_name(domain_clone, &pool).await })
    })
    .ok()??;

    let cert_pem = db_cert.certificate.as_bytes();
    let key_pem = db_cert.private_key.as_bytes();

    let mut cert_reader = BufReader::new(cert_pem);
    let certs_iter = rustls_pemfile::certs(&mut cert_reader);
    let cert_chain: Vec<CertificateDer<'static>> = certs_iter
      .filter_map(|result| result.ok())
      .map(CertificateDer::into_owned)
      .collect();

    if cert_chain.is_empty() {
      error!("No certificates found in PEM for {}", &domain);
      return None;
    }

    let key = match rustls_pemfile::pkcs8_private_keys(&mut std::io::Cursor::new(key_pem))
      .filter_map(Result::ok)
      .next()
    {
      Some(key_data) => {
        let signing_key = rustls::pki_types::PrivateKeyDer::from(key_data);
        match rustls::crypto::ring::sign::any_supported_type(&signing_key) {
          Ok(signing_key) => signing_key,
          Err(_) => {
            error!("Unsupported key type for {}", &domain);
            return None;
          }
        }
      }
      None => {
        error!("No private key found for {}", &domain);
        return None;
      }
    };

    info!("Successfully loaded certificate for {}", domain);
    Some(Arc::new(CertifiedKey::new(cert_chain, key)))
  }
}
