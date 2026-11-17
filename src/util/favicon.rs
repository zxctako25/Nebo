use base64::Engine;
use log::warn;
use std::fs;
use std::path::Path;

pub struct FaviconLoader;

impl FaviconLoader {
    pub fn load(path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        let p = Path::new(path);
        if !p.exists() {
            return None;
        }
        match fs::read(p) {
            Ok(bytes) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                Some(format!("data:image/png;base64,{}", b64))
            }
            Err(e) => {
                warn!("Failed to load favicon {}: {}", p.display(), e);
                None
            }
        }
    }
}
