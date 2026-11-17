mod asn_blocker;
mod config;
mod country_blocker;
mod geo;
mod ip_blocker;
mod ip_list;

use asn_blocker::AsnBlocker;
use config::AntiBotConfig;
use country_blocker::CountryBlocker;
use geo::{normalize_ip, GeoIpService};
use ip_blocker::IpBlocker;
use log::{error, info, warn};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

pub struct AntiBot {
    ip_blocker: IpBlocker,
    country_blocker: CountryBlocker,
    asn_blocker: AsnBlocker,
    geo: Arc<GeoIpService>,
    pub anti_bot_enabled: bool,
    pub limbo_enabled: bool,
    pub captcha_enabled: bool,
    pub captcha_background_count: usize,
    pub captcha_timeout_seconds: u64,
    pub captcha_attempts: u8,
}

#[derive(Debug, Clone)]
pub enum BlockDecision {
    Ip,
    Country { name: String },
    Asn { number: u32, organization: String },
}

impl AntiBot {
    pub async fn load(data_dir: &Path) -> anyhow::Result<Self> {
        let rules_dir = data_dir.join("Nebo");
        fs::create_dir_all(&rules_dir)?;
        let config_path = rules_dir.join("config.yml");
        if !config_path.exists() {
            if let Err(error) = fs::write(&config_path, include_str!("../../../Nebo/config.yml")) {
                warn!("Could not create {}: {}", config_path.display(), error);
            }
        }

        let rules = AntiBotConfig::load(&config_path);
        let geo = GeoIpService::new()?;
        let ip_list = ip_list::IpBlacklist::load(&rules.ips_url).await;
        let ip_blocker = IpBlocker::new(ip_list);
        let country_blocker = CountryBlocker::new(rules.blocked_countries.clone());
        let asn_blocker = AsnBlocker::new(rules.blocked_asns.clone());

        if !rules.ips_url.is_empty() && ip_blocker.len() == 0 {
            error!(
                "ips_url is configured but no IPs were loaded - IP LIST BLOCKING DOES NOT WORK!"
            );
        }

        info!(
            "Nebo anti-bot loaded: {} IPs, {} blocked ASNs, {} blocked countries",
            ip_blocker.len(),
            asn_blocker.len(),
            country_blocker.len()
        );
        info!(
            "Nebo anti-bot country rules: {:?}; ASN rules: {:?}",
            rules.blocked_countries, rules.blocked_asns
        );

        Ok(Self {
            ip_blocker,
            country_blocker,
            asn_blocker,
            geo,
            anti_bot_enabled: rules.anti_bot_enabled,
            limbo_enabled: rules.limbo_enabled,
            captcha_enabled: rules.captcha_enabled,
            captcha_background_count: rules.captcha_background_count,
            captcha_timeout_seconds: rules.captcha_timeout_seconds,
            captcha_attempts: rules.captcha_attempts,
        })
    }

    pub async fn check_ip(&self, ip: IpAddr) -> Option<(IpAddr, BlockDecision)> {
        if !self.anti_bot_enabled {
            return None;
        }
        let ip = normalize_ip(ip);
        let local_test = ip.is_loopback();
        if !local_test && self.ip_blocker.is_blocked(ip) {
            return Some((ip, BlockDecision::Ip));
        }
        let geo = self.geo.lookup(ip).await?;
        let checked_ip = if local_test { geo.ip.unwrap_or(ip) } else { ip };
        if self.ip_blocker.is_blocked(checked_ip) {
            return Some((checked_ip, BlockDecision::Ip));
        }
        if let Some(number) = geo.asn {
            if self.asn_blocker.blocks(number) {
                return Some((
                    checked_ip,
                    BlockDecision::Asn {
                        number,
                        organization: geo.organization.unwrap_or_else(|| "Unknown".to_owned()),
                    },
                ));
            }
        }
        if let Some(code) = geo.country_code {
            if self.country_blocker.blocks(&code) {
                return Some((
                    checked_ip,
                    BlockDecision::Country {
                        name: geo.country_name.unwrap_or(code),
                    },
                ));
            }
        }
        None
    }
}
