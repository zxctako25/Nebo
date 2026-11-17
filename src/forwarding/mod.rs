pub mod velocity;

use crate::config::ProxyConfig;
use log::{error, warn};
use std::sync::Arc;
use uuid::Uuid;

pub struct ForwardingManager {
    pub config: Arc<ProxyConfig>,
    pub modern_enabled: bool,
}

impl ForwardingManager {
    pub fn new(config: Arc<ProxyConfig>) -> anyhow::Result<Self> {
        let modern_enabled = config.forwarding_mode.eq_ignore_ascii_case("modern");
        if modern_enabled
            && config
            .forwarding_secret
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            anyhow::bail!(
                "Forwarding mode is modern but its secret could not be loaded or generated"
            );
        }
        Ok(Self {
            config,
            modern_enabled,
        })
    }

    pub fn create_forwarding_data(
        &self,
        requested_version: i32,
        address: &str,
        uuid: &Uuid,
        name: &str,
    ) -> Option<Vec<u8>> {
        if !self.modern_enabled {
            warn!("Forwarding disabled, but backend requested data — sending empty payload");
            return Some(Vec::new());
        }
        let secret = self.config.forwarding_secret.as_ref()?;
        let data = velocity::create_forwarding_data(secret, requested_version, address, uuid, name);
        if data.is_none() {
            error!("Failed to create forwarding payload for {}", name);
        }
        data
    }
}
