use log::{error, info};
use serde_yaml::Value;
use std::fs;
use std::path::PathBuf;

use super::backend::BackendServer;
use super::limits::Limits;
use super::messages::Messages;
use super::paths::application_data_dir;
use super::secret::load_or_generate_secret;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub data_dir: PathBuf,
    pub bind_address: String,
    pub bind_port: u16,

    pub max_players: i32,
    pub show_online_count: bool,

    pub servers: Vec<BackendServer>,
    pub try_order: Vec<String>,

    pub forwarding_mode: String,
    pub forwarding_secret_file: String,
    pub forwarding_secret: Option<String>,

    pub online_mode: bool,
    pub compression_threshold: i32,
    pub connection_timeout_millis: u64,
    pub read_timeout_seconds: u64,

    pub max_connections_per_ip: i32,
    pub max_total_connections: i32,

    pub unsupported_version_message: String,
    pub backend_offline_message: String,
    pub invalid_username_message: String,
    pub online_mode_unsupported_message: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        let limits = Limits::default();
        let messages = Messages::default();
        Self {
            data_dir: application_data_dir(),
            bind_address: "0.0.0.0".into(),
            bind_port: 25565,
            max_players: limits.max_players,
            show_online_count: limits.show_online_count,
            servers: vec![BackendServer {
                name: "main".into(),
                host: "127.0.0.1".into(),
                port: 25566,
            }],
            try_order: vec!["main".into()],
            forwarding_mode: "modern".into(),
            forwarding_secret_file: "forwarding.secret".into(),
            forwarding_secret: None,
            online_mode: false,
            compression_threshold: limits.compression_threshold,
            connection_timeout_millis: limits.connection_timeout_millis,
            read_timeout_seconds: limits.read_timeout_seconds,
            max_connections_per_ip: limits.max_connections_per_ip,
            max_total_connections: limits.max_total_connections,
            unsupported_version_message: messages.unsupported_version_message,
            backend_offline_message: messages.backend_offline_message,
            invalid_username_message: messages.invalid_username,
            online_mode_unsupported_message: messages.online_mode_unsupported,
        }
    }
}

impl ProxyConfig {
    pub fn load_or_create() -> Self {
        let mut config = ProxyConfig::default();
        let config_path = config.data_dir.join("config.yml");
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    if let Err(e) = config.apply_yaml(&content) {
                        error!("Failed to parse config.yml: {}", e);
                    } else {
                        info!("Config loaded from {}", config_path.display());
                    }
                }
                Err(e) => error!("Failed to read {}: {}", config_path.display(), e),
            }
        } else {
            info!("config.yml not found, creating defaults...");
            config.save_default();
        }

        if config.forwarding_mode.eq_ignore_ascii_case("modern") {
            load_or_generate_secret(&mut config);
        }
        let messages = Messages::load(&config.data_dir);
        config.unsupported_version_message = messages.unsupported_version_message;
        config.backend_offline_message = messages.backend_offline_message;
        config.invalid_username_message = messages.invalid_username;
        config.online_mode_unsupported_message = messages.online_mode_unsupported;

        config
    }

    fn apply_yaml(&mut self, content: &str) -> Result<(), serde_yaml::Error> {
        let value: Value = serde_yaml::from_str(content)?;

        if let Some(net) = value.get("network") {
            if let Some(v) = net.get("bind-address").and_then(|v| v.as_str()) {
                self.bind_address = v.into();
            }
            if let Some(v) = net.get("port").and_then(|v| v.as_u64()) {
                self.bind_port = v as u16;
            }
        }
        if let Some(status) = value.get("status") {
            if let Some(v) = status.get("max-players").and_then(|v| v.as_i64()) {
                self.max_players = v as i32;
            }
            if let Some(v) = status.get("show-online-count").and_then(|v| v.as_bool()) {
                self.show_online_count = v;
            }
        }
        if let Some(backend) = value.get("backend") {
            if let Some(list) = backend.get("servers").and_then(|v| v.as_sequence()) {
                self.servers.clear();
                for s in list {
                    let name = s
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("main")
                        .to_string();
                    let host = s
                        .get("host")
                        .and_then(|v| v.as_str())
                        .unwrap_or("127.0.0.1")
                        .to_string();
                    let port = s.get("port").and_then(|v| v.as_u64()).unwrap_or(25566) as u16;
                    self.servers.push(BackendServer { name, host, port });
                }
            }
            if let Some(order) = backend.get("try-order").and_then(|v| v.as_sequence()) {
                self.try_order = order
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        if let Some(fwd) = value.get("forwarding") {
            if let Some(v) = fwd.get("mode").and_then(|v| v.as_str()) {
                self.forwarding_mode = v.into();
            }
            if let Some(v) = fwd.get("secret-file").and_then(|v| v.as_str()) {
                self.forwarding_secret_file = v.into();
            }
        }
        if let Some(opts) = value.get("options") {
            if let Some(v) = opts.get("online-mode").and_then(|v| v.as_bool()) {
                self.online_mode = v;
            }
            if let Some(v) = opts.get("compression-threshold").and_then(|v| v.as_i64()) {
                self.compression_threshold = v as i32;
            }
            if let Some(v) = opts.get("connection-timeout-ms").and_then(|v| v.as_u64()) {
                self.connection_timeout_millis = v;
            }
            if let Some(v) = opts.get("read-timeout-seconds").and_then(|v| v.as_u64()) {
                self.read_timeout_seconds = v;
            }
        }
        if let Some(limits) = value.get("limits") {
            if let Some(v) = limits
                .get("max-connections-per-ip")
                .and_then(|v| v.as_i64())
            {
                self.max_connections_per_ip = v as i32;
            }
            if let Some(v) = limits.get("max-total-connections").and_then(|v| v.as_i64()) {
                self.max_total_connections = v as i32;
            }
        }
        Ok(())
    }

    fn save_default(&self) {
        let content = r#"
# Proxy listener address and port.
network:
  bind-address: "0.0.0.0"
  port: 25565

# Values displayed in the Minecraft server list.
status:
  max-players: 100
  show-online-count: true

# Backend servers used after a player passes the proxy checks.
# Each server needs a unique name, hostname or IP address, and port.
backend:
  servers:
    - name: "main"
      host: "127.0.0.1" # Backend hostname or IP address.
      port: 25566 # Backend port.
    # Try servers in this order; remaining servers are tried afterward.
  try-order:
    - "main"

# Forwarding settings for backends that support Velocity modern forwarding.
forwarding:
  mode: "modern"
  secret-file: "forwarding.secret"

# General connection and authentication options.
options:
  online-mode: false
  compression-threshold: 256
  connection-timeout-ms: 10000
  read-timeout-seconds: 30

# Connection limits.
limits:
  max-connections-per-ip: 0 # 0 disables the per-IP limit.
  max-total-connections: 1000

"#;
        let path = self.data_dir.join("config.yml");
        if let Err(e) = fs::write(&path, content) {
            error!("Failed to create {}: {}", path.display(), e);
        }
    }
}
