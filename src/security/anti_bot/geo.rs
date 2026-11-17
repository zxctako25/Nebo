use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;

const API_BASE: &str = "https://ipwho.is";
const API_DAILY_BUDGET: u16 = 900;
const CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const ERROR_CACHE_TTL: Duration = Duration::from_secs(30);
const MAX_GEO_CACHE_ENTRIES: usize = 4096;

#[derive(Clone, Debug)]
pub(super) struct GeoIpRecord {
    pub ip: Option<IpAddr>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub asn: Option<u32>,
    pub organization: Option<String>,
}

struct CachedLookup {
    result: Option<GeoIpRecord>,
    expires_at: Instant,
}

struct QuotaWindow {
    started_at: Instant,
    requests: u16,
}

#[derive(Deserialize)]
struct ApiResponse {
    success: bool,
    #[serde(default)]
    ip: Option<IpAddr>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    connection: Option<ApiConnection>,
}

#[derive(Deserialize)]
struct ApiConnection {
    #[serde(default)]
    asn: Option<u32>,
    #[serde(default)]
    org: Option<String>,
}

pub(super) struct GeoIpService {
    client: reqwest::Client,
    cache: RwLock<HashMap<IpAddr, CachedLookup>>,
    quota: Mutex<QuotaWindow>,
}

impl GeoIpService {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
            ))
            .build()?;
        info!("GeoIP and ASN lookups use the HTTPS ipwho.is API; no MMDB files required");
        Ok(Arc::new(Self {
            client,
            cache: RwLock::new(HashMap::new()),
            quota: Mutex::new(QuotaWindow {
                started_at: Instant::now(),
                requests: 0,
            }),
        }))
    }

    pub async fn lookup(&self, ip: IpAddr) -> Option<GeoIpRecord> {
        let ip = normalize_ip(ip);
        if let Some(cached) = self.cache.read().await.get(&ip) {
            if cached.expires_at > Instant::now() {
                return cached.result.clone();
            }
        }

        let query_ip = if ip.is_loopback() { None } else { Some(ip) };
        let result = if self.reserve_request().await {
            self.fetch(query_ip).await
        } else {
            warn!(
                "GeoIP API daily request budget reached; lookup skipped for {}",
                ip
            );
            None
        };
        let ttl = if result.is_some() {
            CACHE_TTL
        } else {
            ERROR_CACHE_TTL
        };
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        if cache.len() >= MAX_GEO_CACHE_ENTRIES {
            cache.retain(|_, entry| entry.expires_at > now);
            if cache.len() >= MAX_GEO_CACHE_ENTRIES {
                let remove_count = cache.len() - MAX_GEO_CACHE_ENTRIES + 1;
                let expired_keys: Vec<_> = cache.keys().take(remove_count).copied().collect();
                for key in expired_keys {
                    cache.remove(&key);
                }
            }
        }
        cache.insert(
            ip,
            CachedLookup {
                result: result.clone(),
                expires_at: now + ttl,
            },
        );
        result
    }

    async fn reserve_request(&self) -> bool {
        let mut quota = self.quota.lock().await;
        if quota.started_at.elapsed() >= Duration::from_secs(24 * 60 * 60) {
            quota.started_at = Instant::now();
            quota.requests = 0;
        }
        if quota.requests >= API_DAILY_BUDGET {
            return false;
        }
        quota.requests += 1;
        true
    }

    async fn fetch(&self, ip: Option<IpAddr>) -> Option<GeoIpRecord> {
        let url = ip.map_or_else(|| API_BASE.to_owned(), |ip| format!("{API_BASE}/{ip}"));
        let label = ip.map_or_else(
            || "local client / public egress".to_owned(),
            |ip| ip.to_string(),
        );
        let response = match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => response,
            Ok(response) => {
                warn!(
                    "GeoIP API returned HTTP {} for {}",
                    response.status(),
                    label
                );
                return None;
            }
            Err(error) => {
                warn!("GeoIP API request failed for {}: {}", label, error);
                return None;
            }
        };
        let data = match response.json::<ApiResponse>().await {
            Ok(data) => data,
            Err(error) => {
                warn!("Invalid GeoIP API response for {}: {}", label, error);
                return None;
            }
        };
        if !data.success {
            warn!(
                "GeoIP API could not resolve {}: {}",
                label,
                data.message.as_deref().unwrap_or("unknown error")
            );
            return None;
        }
        let connection = data.connection.unwrap_or(ApiConnection {
            asn: None,
            org: None,
        });
        Some(GeoIpRecord {
            ip: data.ip,
            country_code: data.country_code.map(|code| code.to_ascii_uppercase()),
            country_name: data.country,
            asn: connection.asn,
            organization: connection.org,
        })
    }
}

pub(super) fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ipv6) => ipv6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ipv6)),
        ipv4 => ipv4,
    }
}
