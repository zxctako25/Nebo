use log::{error, info, warn};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

use super::proxy::ProxyConfig;

pub(crate) fn load_or_generate_secret(config: &mut ProxyConfig) {
    let configured_path = PathBuf::from(&config.forwarding_secret_file);
    let path = if configured_path.is_absolute() {
        configured_path
    } else {
        config.data_dir.join(configured_path)
    };
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(s) if !s.trim().is_empty() => {
                config.forwarding_secret = Some(s.trim().to_string());
                info!("Forwarding secret loaded from {}", path.display());
            }
            Ok(_) => error!(
                "Forwarding secret file {} is empty; refusing to replace it",
                path.display()
            ),
            Err(e) => error!(
                "Cannot read forwarding secret {}: {}; refusing to replace it",
                path.display(),
                e
            ),
        }
        return;
    }
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    let secret = hex::encode(bytes);
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            error!(
                "Failed to create secret directory {}: {}",
                parent.display(),
                e
            );
            return;
        }
    }
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => match file.write_all(secret.as_bytes()) {
            Ok(()) => {
                config.forwarding_secret = Some(secret.clone());
                warn!("============================================================");
                warn!(
                    "Generated new forwarding secret and saved to: {}",
                    path.display()
                );
                warn!(
                    "Put it into paper.yml velocity-support.secret: '{}'",
                    secret
                );
                warn!("============================================================");
            }
            Err(e) => error!(
                "Failed to write forwarding secret to {}: {}; refusing to continue with a replacement",
                path.display(),
                e
            ),
        },
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            match fs::read_to_string(&path) {
                Ok(existing) if !existing.trim().is_empty() => {
                    config.forwarding_secret = Some(existing.trim().to_string());
                    info!("Forwarding secret loaded from {}", path.display());
                }
                Ok(_) => error!(
                    "Forwarding secret file {} is empty; refusing to replace it",
                    path.display()
                ),
                Err(read_error) => error!(
                    "Cannot read forwarding secret {}: {}",
                    path.display(),
                    read_error
                ),
            }
        }
        Err(e) => error!(
            "Failed to create forwarding secret {}: {}",
            path.display(),
            e
        ),
    }
}
