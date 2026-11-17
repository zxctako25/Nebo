use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendServer {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl Default for BackendServer {
    fn default() -> Self {
        Self {
            name: "main".into(),
            host: "127.0.0.1".into(),
            port: 25566,
        }
    }
}
