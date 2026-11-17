use log::warn;
use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub(super) struct AntiBotConfig {
    pub blocked_countries: HashSet<String>,
    pub blocked_asns: HashSet<u32>,
    pub ips_url: Vec<String>,
    pub anti_bot_enabled: bool,
    pub limbo_enabled: bool,
    pub captcha_enabled: bool,
    pub captcha_background_count: usize,
    pub captcha_timeout_seconds: u64,
    pub captcha_attempts: u8,
}

impl AntiBotConfig {
    pub fn load(path: &Path) -> Self {
        let mut config = Self {
            blocked_countries: HashSet::new(),
            blocked_asns: HashSet::new(),
            ips_url: Vec::new(),
            anti_bot_enabled: true,
            limbo_enabled: true,
            captcha_enabled: true,
            captcha_background_count: 1,
            captcha_timeout_seconds: 30,
            captcha_attempts: 3,
        };
        let Ok(text) = fs::read_to_string(path) else {
            warn!("Nebo anti-bot config missing: {}", path.display());
            return config;
        };
        let doc = match serde_yaml::from_str::<Value>(&text) {
            Ok(doc) => doc,
            Err(error) => {
                warn!("Invalid Nebo anti-bot config {}: {}", path.display(), error);
                return config;
            }
        };

        config.blocked_countries = read_values(&doc, "blocked_country")
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_ascii_uppercase()))
            .filter(|s| s.len() == 2 && s.bytes().all(|b| b.is_ascii_uppercase()))
            .collect();
        config.blocked_asns = read_values(&doc, "blocked_asns")
            .into_iter()
            .filter_map(|v| {
                v.as_u64().or_else(|| {
                    v.as_str().and_then(|s| {
                        s.trim()
                            .trim_start_matches(['A', 'a'])
                            .trim_start_matches(['S', 's'])
                            .parse()
                            .ok()
                    })
                })
            })
            .filter_map(|v| u32::try_from(v).ok())
            .collect();
        config.ips_url = match doc.get("ips_url") {
            Some(Value::String(url)) => vec![url.clone()],
            Some(Value::Sequence(urls)) => urls
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            _ => Vec::new(),
        };
        if let Some(value) = doc.get("anti_bot").and_then(|v| v.as_bool()) {
            config.anti_bot_enabled = value;
        }
        if let Some(value) = doc.get("limbo").and_then(|v| v.get("enabled")).and_then(|v| v.as_bool()) {
            config.limbo_enabled = value;
        }
        if let Some(value) = doc.get("captcha").and_then(|v| v.get("enabled")).and_then(|v| v.as_bool()) {
            config.captcha_enabled = value;
        }
        if let Some(value) = doc.get("captcha").and_then(|v| v.get("background-count")).and_then(|v| v.as_u64()) {
            config.captcha_background_count = value.clamp(1, 100) as usize;
        }
        if let Some(value) = doc.get("captcha").and_then(|v| v.get("timeout-seconds")).and_then(|v| v.as_u64()) {
            config.captcha_timeout_seconds = value.clamp(1, 600);
        }
        if let Some(value) = doc.get("captcha").and_then(|v| v.get("attempts")).and_then(|v| v.as_u64()) {
            config.captcha_attempts = value.clamp(1, 20) as u8;
        }
        config
    }
}

fn read_values(doc: &Value, key: &str) -> Vec<Value> {
    match doc.get(key) {
        Some(Value::Sequence(values)) => values.clone(),
        Some(Value::String(value)) => value
            .split([',', ' ', '\t', '\r', '\n'])
            .filter(|part| !part.is_empty())
            .map(|part| Value::String(part.trim().to_owned()))
            .collect(),
        _ => Vec::new(),
    }
}
