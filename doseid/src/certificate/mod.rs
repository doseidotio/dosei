use cached::{Cached, TimedCache};
use chrono::{DateTime, Utc};
use instant_acme::{
  Account as AcmeAccount, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, Order,
  OrderStatus,
};
use once_cell::sync::Lazy;
use rcgen::{CertificateParams, DistinguishedName};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod route;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Certificate {
  pub id: Uuid,
  pub domain_name: String,
  pub certificate: String,
  pub private_key: String,
  pub expires_at: DateTime<Utc>,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub(crate) struct PendingCertificate {
  pub domain_name: String,
  pub owner_id: Uuid,
  pub token: String,
  pub token_value: String,
  pub order: Arc<Mutex<Order>>,
}

const CACHE_LIFESPAN: u64 = 600;
const INTERNAL_CHECK_SPAN: u64 = 5; // 5 seconds
const RENEWAL_CHECK_SPAN: u64 = 86400; // 24 hours
const EXTERNAL_MAX_CHECKS: u64 = 10;

pub struct CertificateManager {
  pub internal_pending_certificate_cache: Arc<Mutex<TimedCache<String, PendingCertificate>>>,
  pub http1_challenge_token_cache: Arc<Mutex<TimedCache<String, String>>>,
}

pub static CERTIFICATE_MANAGER: Lazy<CertificateManager> = Lazy::new(|| CertificateManager {
  internal_pending_certificate_cache: Arc::new(Mutex::new(TimedCache::with_lifespan(
    CACHE_LIFESPAN,
  ))),
  http1_challenge_token_cache: Arc::new(Mutex::new(TimedCache::with_lifespan(CACHE_LIFESPAN))),
});

pub async fn start_certificate_server(pg_pool: &Arc<Pool<Postgres>>) -> anyhow::Result<()> {
  info!("Doseid Certificate Server Running");

  let pool_clone = Arc::clone(pg_pool);
  // Start Internal Check Loop
  tokio::spawn(async move {
    let mut interval = interval(Duration::from_secs(INTERNAL_CHECK_SPAN));

    loop {
      interval.tick().await;

      let mut certificates_to_check = Vec::new();
      {
        let cache_lock = CERTIFICATE_MANAGER
          .internal_pending_certificate_cache
          .lock()
          .await;
        for (_, (_, cert)) in cache_lock.get_store().iter() {
          certificates_to_check.push(cert.clone());
        }
      }
      if !certificates_to_check.is_empty() {
        info!("Active pending certificates in cache:");
      }
      for cert in certificates_to_check {
        info!("{}", cert.domain_name);
        let cert_pool = Arc::clone(&pool_clone);
        tokio::spawn(async move {
          let mut opts = ResolverOpts::default();
          opts.cache_size = 0;
          opts.negative_max_ttl = Some(Duration::from_secs(0));
          opts.positive_max_ttl = Some(Duration::from_secs(0));

          let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), opts);
          let lookup_result = resolver.ipv4_lookup(&cert.domain_name).await;

          if let Err(e) = &lookup_result {
            error!("DNS lookup failed for {}: {:?}", cert.domain_name, e);
            return;
          }

          // Get the first IP address
          let response = lookup_result.unwrap();
          let address = match response.iter().next() {
            Some(addr) => addr,
            None => {
              error!("No IP address found for domain: {}", cert.domain_name);
              return;
            }
          };

          // Build the HTTP client
          let client_result = reqwest::Client::builder().no_proxy().build();

          if let Err(e) = &client_result {
            error!("Failed to build HTTP client: {:?}", e);
            return;
          }

          let client = client_result.unwrap();

          // Create URL for verification
          let url = format!(
            "http://{}/.well-known/acme-challenge/{}",
            address, cert.token
          );

          // Send HTTP request
          let response_result = client.get(&url).send().await;

          if let Err(e) = &response_result {
            error!("Request failed for {}: {:?}", cert.domain_name, e);
            return;
          }

          let response = response_result.unwrap();

          // Get the response text
          let text_result = response.text().await;

          if let Err(e) = &text_result {
            error!(
              "Failed to get response text for {}: {:?}",
              cert.domain_name, e
            );
            return;
          }

          let response_text = text_result.unwrap();

          // Verify the token
          if response_text == cert.token_value {
            info!(
              "Certificate verification successful for {}: All good go for external check!",
              cert.domain_name
            );
            CertificateManager::external_check(cert.clone(), cert_pool);
            let internal_pending_certificate_cache =
              Arc::clone(&CERTIFICATE_MANAGER.internal_pending_certificate_cache);
            {
              let mut cache = internal_pending_certificate_cache.lock().await;
              cache.cache_remove(&cert.domain_name);
            }
          } else {
            warn!(
              "Certificate verification failed for {}: Token mismatch",
              cert.domain_name
            );
          }
        });
      }
    }
  });

  // Add this new task for certificate renewal checks
  let renewal_pool_clone = Arc::clone(pg_pool);
  tokio::spawn(async move {
    let mut renewal_interval = interval(Duration::from_secs(RENEWAL_CHECK_SPAN));

    loop {
      renewal_interval.tick().await;

      info!("Running certificate renewal check");
      if let Err(e) = Certificate::check_for_renewals(&renewal_pool_clone).await {
        error!("Error during certificate renewal check: {:?}", e);
      }
    }
  });
  Ok(())
}

impl CertificateManager {
  pub fn external_check(pending_certificate: PendingCertificate, pg_pool: Arc<Pool<Postgres>>) {
    let order = Arc::clone(&pending_certificate.order);

    let mut attempts = 1;
    let mut backoff_duration = Duration::from_millis(250);
    tokio::spawn(async move {
      loop {
        sleep(backoff_duration).await;
        let mut order_guard = order.lock().await;

        let authorizations = order_guard.authorizations().await.unwrap();
        let authorization = &authorizations
          .first()
          .ok_or_else(|| anyhow::Error::msg("authorization not found"))
          .unwrap();
        let challenge = authorization
          .challenges
          .iter()
          .find(|ch| ch.r#type == ChallengeType::Http01)
          .ok_or_else(|| anyhow::Error::msg("http-01 challenge not found"))
          .unwrap();
        order_guard
          .set_challenge_ready(&challenge.url)
          .await
          .unwrap();

        let order_state = order_guard.refresh().await.unwrap();
        match order_state.status {
          OrderStatus::Ready => {
            drop(order_guard);
            match Certificate::new(pending_certificate, Arc::clone(&pg_pool)).await {
              Ok(_) => {}
              Err(_) => error!("Something went wrong when generating CERT"),
            };
            break;
          }
          order_status => {
            error!("Order Status: {:?}", order_status);
            error!("Full Order State: {:?}", order_state);
            backoff_duration *= 4;
            attempts += 1;

            if EXTERNAL_MAX_CHECKS <= attempts {
              error!("Order is not yet ready after {EXTERNAL_MAX_CHECKS} attempts, Giving up.");
              break;
            }
            info!("Order is not ready, waiting {backoff_duration:?}");
          }
        }
      }
    });
  }

  pub async fn get_http01_challenge_token_value(token: String) -> Option<String> {
    let http1_challenge_token_cache = Arc::clone(&CERTIFICATE_MANAGER.http1_challenge_token_cache);
    {
      let mut cache = http1_challenge_token_cache.lock().await;
      if let Some(value) = cache.cache_get(&token) {
        let token_value = value.clone();
        return cache.cache_set(token, token_value);
      }
    }
    None
  }
}

impl Certificate {
  pub async fn request(owner_id: Uuid, domain_name: &str) -> anyhow::Result<()> {
    let server_url = LetsEncrypt::Production.url().to_string();

    let new_account_info = NewAccount {
      contact: &[],
      terms_of_service_agreed: true,
      only_return_existing: false,
    };

    let (_account, credentials) = AcmeAccount::create(&new_account_info, &server_url, None).await?;

    let mut order = AcmeAccount::from_credentials(credentials)
      .await?
      .new_order(&NewOrder {
        identifiers: &[Identifier::Dns(domain_name.to_string())],
      })
      .await?;
    let authorizations = order.authorizations().await?;
    let authorization = &authorizations
      .first()
      .ok_or_else(|| anyhow::Error::msg("authorization not found"))?;
    let challenge = authorization
      .challenges
      .iter()
      .find(|ch| ch.r#type == ChallengeType::Http01)
      .ok_or_else(|| anyhow::Error::msg("http-01 challenge not found"))?;

    let http1_challenge_token_cache = Arc::clone(&CERTIFICATE_MANAGER.http1_challenge_token_cache);
    {
      let mut cache = http1_challenge_token_cache.lock().await;
      cache.cache_set(
        challenge.token.clone(),
        order.key_authorization(challenge).as_str().to_string(),
      );
    }

    let internal_pending_certificate_cache =
      Arc::clone(&CERTIFICATE_MANAGER.internal_pending_certificate_cache);
    {
      let mut cache = internal_pending_certificate_cache.lock().await;
      cache.cache_set(
        domain_name.to_string(),
        PendingCertificate {
          domain_name: domain_name.to_string(),
          owner_id,
          token: challenge.token.clone(),
          token_value: order.key_authorization(challenge).as_str().to_string(),
          order: Arc::new(Mutex::new(order)),
        },
      );
    }
    Ok(())
  }

  async fn new(
    pending_certificate: PendingCertificate,
    pg_pool: Arc<Pool<Postgres>>,
  ) -> anyhow::Result<Certificate> {
    let order = Arc::clone(&pending_certificate.order);
    let certificate = {
      let mut params = CertificateParams::new(vec![pending_certificate.domain_name.to_owned()]);
      params.distinguished_name = DistinguishedName::new();
      rcgen::Certificate::from_params(params)?
    };
    let signing_request = certificate.serialize_request_der()?;
    let mut order = order.lock().await;
    order.finalize(&signing_request).await?;

    let cert_chain_pem = loop {
      match order.certificate().await? {
        Some(cert_chain_pem) => break cert_chain_pem,
        None => sleep(Duration::from_secs(1)).await,
      }
    };

    let mut certificates: Vec<String> = cert_chain_pem
      .split("-----END CERTIFICATE-----")
      .map(|cert| format!("{}-----END CERTIFICATE-----", cert))
      .collect();
    certificates.pop();

    let mut expires_at = Utc::now();
    if let Ok(cert) = openssl::x509::X509::from_pem(certificates[0].as_bytes()) {
      let not_after_str = cert.not_after().to_string().replace("GMT", "+0000");
      if let Ok(not_after) = DateTime::parse_from_str(&not_after_str, "%b %d %H:%M:%S %Y %z") {
        expires_at = not_after.with_timezone(&Utc);
      }
    }

    let certificate = Certificate {
      id: Uuid::new_v4(),
      domain_name: pending_certificate.domain_name.to_string(),
      certificate: certificates[0].to_string(),
      private_key: certificate.serialize_private_key_pem(),
      expires_at,
      owner_id: pending_certificate.owner_id,
      updated_at: Utc::now(),
      created_at: Utc::now(),
    };
    let certificate = sqlx::query_as!(
        Certificate,
        "INSERT INTO certificate (id, domain_name, certificate, private_key, expires_at, owner_id, updated_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
        certificate.id,
        certificate.domain_name,
        certificate.certificate,
        certificate.private_key,
        certificate.expires_at,
        certificate.owner_id,
        certificate.updated_at,
        certificate.created_at,
    )
      .fetch_one(&*pg_pool)
      .await?;
    // TODO: Send email and notify.
    info!("Created certificate: {:?}", certificate.domain_name);
    Ok(certificate)
  }

  pub async fn renew(domain_name: &str, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    // Get the existing certificate to get the owner email
    let existing_cert = Self::get_by_domain_name(domain_name.to_string(), pg_pool)
      .await?
      .ok_or_else(|| anyhow::Error::msg("Certificate not found for renewal"))?;

    // Request a new certificate using the same process as for new certificates
    Self::request(existing_cert.owner_id, domain_name).await?;

    info!("Certificate renewal requested for: {}", domain_name);
    Ok(())
  }

  // Add a method to check for certificates nearing expiration
  pub async fn check_for_renewals(pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    // Define the renewal threshold (e.g., 30 days before expiration)
    let renewal_threshold = Utc::now() + chrono::Duration::days(30);

    // Find certificates nearing expiration
    let certificates = sqlx::query_as!(
      Certificate,
      "SELECT * FROM certificate WHERE expires_at < $1",
      renewal_threshold
    )
    .fetch_all(pg_pool)
    .await?;

    for cert in certificates {
      info!(
        "Certificate for {} expires soon, initiating renewal",
        cert.domain_name
      );
      match Self::renew(&cert.domain_name, pg_pool).await {
        Ok(_) => info!("Renewal initiated for {}", cert.domain_name),
        Err(e) => error!(
          "Failed to renew certificate for {}: {:?}",
          cert.domain_name, e
        ),
      }
    }

    Ok(())
  }

  pub async fn get_by_domain_name(
    domain_name: String,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Option<Certificate>> {
    let certificate = sqlx::query_as!(
      Certificate,
      "SELECT * FROM certificate WHERE domain_name = $1",
      domain_name
    )
    .fetch_optional(pg_pool)
    .await?;
    Ok(certificate)
  }

  pub async fn get_by_owner_id(
    owner_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Vec<Certificate>> {
    Ok(
      sqlx::query_as!(
        Certificate,
        "SELECT * FROM certificate WHERE owner_id = $1",
        owner_id
      )
      .fetch_all(pg_pool)
      .await?,
    )
  }
}
