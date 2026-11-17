use std::path::Path;

#[derive(Debug, Clone)]
pub struct LimboTexts {
    pub prefix: String,
    pub checking_chat: String,
    pub checking_title: String,
    pub checking_subtitle: String,
    pub captcha_chat: String,
    pub wrong_captcha_chat: String,
    pub captcha_title: String,
    pub captcha_subtitle: String,
    pub successful: String,
    pub bot_detected: String,
    pub timeout: String,
    pub anti_bot_country: String,
    pub anti_bot_asn: String,
    pub anti_bot_ip: String,
    pub invalid_username: String,
    pub online_mode_unsupported: String,
    pub limbo_action_bar: String,
}

impl LimboTexts {
    pub fn load(data_dir: &Path) -> Self {
        let language = std::fs::read_to_string(data_dir.join("Nebo").join("config.yml"))
            .ok()
            .and_then(|s| serde_yaml::from_str::<serde_yaml::Value>(&s).ok())
            .and_then(|v| v.get("messages")?.get("language")?.as_str().map(str::to_owned))
            .filter(|s| matches!(s.as_str(), "ru" | "en"))
            .unwrap_or_else(|| "ru".to_owned());
        let path = data_dir.join("Nebo").join("messages").join(format!("messages_{language}.yml"));
        let mut texts = Self::default();
        let Ok(source) = std::fs::read_to_string(&path) else {
            log::warn!(
                "Message file not found at {}; using built-in defaults",
                path.display()
            );
            return texts;
        };
        let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(&source) else {
            log::warn!(
                "Could not parse message file at {}; using built-in defaults",
                path.display()
            );
            return texts;
        };
        texts.checking_chat = get(&root, "checking-chat", &texts.checking_chat);
        texts.checking_title = get(&root, "checking-title", &texts.checking_title);
        texts.checking_subtitle = get(&root, "checking-subtitle", &texts.checking_subtitle);
        texts.captcha_chat = get(&root, "captcha-prompt", &texts.captcha_chat);
        texts.captcha_title = get(&root, "captcha-title", &texts.captcha_title);
        texts.captcha_subtitle = get(&root, "captcha-subtitle", &texts.captcha_subtitle);
        texts.wrong_captcha_chat = get(&root, "captcha-wrong", &texts.wrong_captcha_chat);
        texts.successful = get(&root, "captcha-success", &texts.successful);
        texts.bot_detected = get(&root, "bot-detected", &texts.bot_detected);
        texts.timeout = get(&root, "limbo-timeout", &texts.timeout);
        texts.anti_bot_country = get(&root, "anti-bot-country", &texts.anti_bot_country);
        texts.anti_bot_asn = get(&root, "anti-bot-asn", &texts.anti_bot_asn);
        texts.anti_bot_ip = get(&root, "anti-bot-ip", &texts.anti_bot_ip);
        texts.invalid_username = get(&root, "invalid-username", &texts.invalid_username);
        texts.online_mode_unsupported = get(&root, "online-mode-unsupported", &texts.online_mode_unsupported);
        texts.limbo_action_bar = get(&root, "limbo-action-bar", &texts.limbo_action_bar);
        log::info!("Player messages loaded from {}", path.display());
        texts
    }

    pub fn render(&self, value: &str) -> String {
        let expanded = value.replace("{PRFX}", &self.prefix).replace("{NL}", "\n");
        color_codes(&expanded)
    }
}

impl Default for LimboTexts {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            checking_chat: "&#FF4F00&l▶ &fОжидайте завершения проверки... Пожалуйста, не двигайтесь.".into(),
            checking_title: "{PRFX}".into(),
            checking_subtitle: "&fИдет проверка...".into(),
            captcha_chat: "&#FF4F00&l▶ &fВведите цифры с картинки в чат! Для открытия чата, &#FFA200нажмите <T>".into(),
            wrong_captcha_chat: "&#FF4F00&l▶ &fВы ввели символы с картинки неправильно Пожалуйста попробуйте еще раз.".into(),
            captcha_title: "&bПроверка безопасности".into(),
            captcha_subtitle: "&fВведите код с картинки".into(),
            successful: "&aОтвет правильный. Подключаем к серверу...".into(),
            bot_detected: "&cПроверка подключения не пройдена. Вы отключены как бот.".into(),
            timeout: "&cВремя проверки истекло. Подключитесь и попробуйте снова.".into(),
            anti_bot_country: "&cВаша страна {country} заблокирована на этом сервере.".into(),
            anti_bot_asn: "&cВаш провайдер {organization} (ASN {asn}) заблокирован на этом сервере.".into(),
            anti_bot_ip: "&cВаш IP-адрес {ip} заблокирован на этом сервере.".into(),
            invalid_username: "&cНекорректный никнейм.".into(),
            online_mode_unsupported: "&cАвторизация online-mode пока не поддерживается.".into(),
            limbo_action_bar: "&aВы на Limbo. Ожидайте проверки подключения.".into(),
        }
    }
}

fn get(section: &serde_yaml::Value, key: &str, fallback: &str) -> String {
    section
        .get(key)
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

pub fn color_codes(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' && i + 7 < chars.len() && chars[i + 1] == '#' {
            let hex: String = chars[i + 2..i + 8].iter().collect();
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                out.push('§');
                out.push('x');
                for c in hex.chars() {
                    out.push('§');
                    out.push(c);
                }
                i += 8;
                continue;
            }
        }
        if chars[i] == '&' && i + 1 < chars.len() {
            if chars[i + 1] == 'x' && i + 13 < chars.len() {
                out.push('§');
                for c in &chars[i + 1..=i + 13] {
                    out.push(if *c == '&' { '§' } else { *c });
                }
                i += 14;
                continue;
            }
            if "0123456789abcdefklmnor".contains(chars[i + 1].to_ascii_lowercase()) {
                out.push('§');
                out.push(chars[i + 1].to_ascii_lowercase());
                i += 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}
