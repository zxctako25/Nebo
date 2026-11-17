#[derive(Debug, Clone)]
pub(crate) struct Messages {
    pub unsupported_version_message: String,
    pub backend_offline_message: String,
    pub invalid_username: String,
    pub online_mode_unsupported: String,
}

impl Default for Messages {
    fn default() -> Self {
        Self {
            unsupported_version_message: "§cUnsupported version.".into(),
            backend_offline_message: "§cBackend server is offline.".into(),
            invalid_username: "§cInvalid username.".into(),
            online_mode_unsupported: "§cOnline mode is not supported.".into(),
        }
    }
}

impl Messages {
    pub fn load(data_dir: &std::path::Path) -> Self {
        let messages_dir = data_dir.join("Nebo").join("messages");
        if let Err(error) = std::fs::create_dir_all(&messages_dir) {
            log::warn!("Could not create {}: {}", messages_dir.display(), error);
        }
        for (name, contents) in [
            ("messages_ru.yml", include_str!("../../Nebo/messages/messages_ru.yml")),
            ("messages_en.yml", include_str!("../../Nebo/messages/messages_en.yml")),
        ] {
            let path = messages_dir.join(name);
            if !path.exists() {
                if let Err(error) = std::fs::write(&path, contents) {
                    log::warn!("Could not create {}: {}", path.display(), error);
                }
            }
        }
        let language = std::fs::read_to_string(data_dir.join("Nebo").join("config.yml"))
            .ok()
            .and_then(|text| serde_yaml::from_str::<serde_yaml::Value>(&text).ok())
            .and_then(|root| root.get("messages")?.get("language")?.as_str().map(str::to_owned))
            .filter(|language| matches!(language.as_str(), "ru" | "en"))
            .unwrap_or_else(|| "ru".to_owned());
        let path = data_dir.join("Nebo").join("messages").join(format!("messages_{language}.yml"));
        let Ok(text) = std::fs::read_to_string(path) else { return Self::default() };
        let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(&text) else { return Self::default() };
        let get = |key: &str, fallback: &str| root.get(key).and_then(|v| v.as_str()).unwrap_or(fallback).to_owned();
        let defaults = Self::default();
        Self {
            unsupported_version_message: get("unsupported-version", &defaults.unsupported_version_message),
            backend_offline_message: get("backend-offline", &defaults.backend_offline_message),
            invalid_username: get("invalid-username", &defaults.invalid_username),
            online_mode_unsupported: get("online-mode-unsupported", &defaults.online_mode_unsupported),
        }
    }
}
