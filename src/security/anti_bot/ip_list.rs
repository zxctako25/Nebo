use super::geo::normalize_ip;
use log::{info, warn};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

const MAX_REMOTE_LIST_BYTES: u64 = 16 * 1024 * 1024;
const DOWNLOAD_ATTEMPTS: u32 = 3;

#[derive(Default)]
pub(super) struct IpBlacklist {
    ipv4: HashSet<u32>,
    ipv6: HashSet<u128>,
}

impl IpBlacklist {
    pub async fn load(urls: &[String]) -> Self {
        let mut list = Self::default();
        if urls.is_empty() {
            return list;
        }
        list.load_remote(urls).await;
        list.compact();
        list
    }

    pub fn len(&self) -> usize {
        self.ipv4.len() + self.ipv6.len()
    }

    pub fn contains(&self, ip: &IpAddr) -> bool {
        match normalize_ip(*ip) {
            IpAddr::V4(ip) => self.ipv4.contains(&u32::from(ip)),
            IpAddr::V6(ip) => self.ipv6.contains(&u128::from(ip)),
        }
    }

    fn insert(&mut self, ip: IpAddr) {
        match normalize_ip(ip) {
            IpAddr::V4(ip) => {
                self.ipv4.insert(u32::from(ip));
            }
            IpAddr::V6(ip) => {
                self.ipv6.insert(u128::from(ip));
            }
        }
    }

    fn compact(&mut self) {
        self.ipv4.shrink_to_fit();
        self.ipv6.shrink_to_fit();
    }

    async fn load_remote(&mut self, urls: &[String]) {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; NeboAntiBot/1.0)")
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                warn!("Could not initialize IP list downloader: {}", error);
                return;
            }
        };

        for url in urls {
            match download(&client, url).await {
                Some(body) => {
                    let before = self.len();
                    extract_ips(&body, self);
                    info!("Loaded {} new IPs from {}", self.len() - before, url);
                }
                None => {
                    warn!("IP list {} could not be downloaded", url);
                }
            }
        }
    }
}

async fn download(client: &reqwest::Client, url: &str) -> Option<String> {
    for attempt in 1..=DOWNLOAD_ATTEMPTS {
        match fetch(client, url).await {
            Ok(body) => return Some(body),
            Err(error) => {
                warn!(
                    "Download of {} failed (attempt {}/{}): {}",
                    url, attempt, DOWNLOAD_ATTEMPTS, error
                );
                if attempt < DOWNLOAD_ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
    None
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    if response
        .content_length()
        .is_some_and(|n| n > MAX_REMOTE_LIST_BYTES)
    {
        return Err("list too large".into());
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REMOTE_LIST_BYTES {
        return Err("list too large".into());
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn extract_ips(body: &str, out: &mut IpBlacklist) {
    let is_ip_char = |c: char| c.is_ascii_hexdigit() || matches!(c, '.' | ':' | '[' | ']');
    for token in body.split(|c: char| !is_ip_char(c)) {
        if token.is_empty() {
            continue;
        }
        if let Ok(socket) = token.parse::<SocketAddr>() {
            out.insert(socket.ip());
            continue;
        }
        if let Ok(ip) = token.trim_matches(['[', ']']).parse::<IpAddr>() {
            if !ip.is_unspecified() {
                out.insert(ip);
            }
        }
    }
}
