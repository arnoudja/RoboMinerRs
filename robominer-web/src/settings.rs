use std::env;
use std::path::{Path, PathBuf};

use robominer_db::LegacyAppConfig;

pub struct WebSettings {
    pub host: String,
    pub port: String,
    pub static_root: PathBuf,
    pub session_secret: Option<String>,
    pub session_ttl_secs: Option<String>,
    pub session_ttl_hours: Option<String>,
    /// `None` means unset → Secure cookies stay off (set `securecookies 1` for HTTPS).
    pub secure_cookies: Option<bool>,
    pub allow_signup: bool,
    pub trust_proxy: bool,
    pub allow_insecure_dev_secret: bool,
}

pub fn web_settings(config: &LegacyAppConfig, default_static_root: &Path) -> WebSettings {
    WebSettings {
        host: env::var("HOST")
            .ok()
            .or_else(|| config.host.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        port: env::var("PORT")
            .ok()
            .or_else(|| config.port.clone())
            .unwrap_or_else(|| "8080".to_string()),
        static_root: env::var("ROBOMINER_WEB_ROOT")
            .map(PathBuf::from)
            .ok()
            .or_else(|| config.web_root.as_ref().map(PathBuf::from))
            .unwrap_or_else(|| default_static_root.to_path_buf()),
        session_secret: env::var("ROBOMINER_SESSION_SECRET")
            .ok()
            .or_else(|| config.session_secret.clone()),
        // TTL env vars only here; file keys are applied in `prepare_server_config`
        // so env hours still beat config secs (see `resolve_session_ttl_secs`).
        session_ttl_secs: env::var("ROBOMINER_SESSION_TTL_SECS").ok(),
        session_ttl_hours: env::var("ROBOMINER_SESSION_TTL_HOURS").ok(),
        secure_cookies: parse_optional_bool_setting(
            env::var("ROBOMINER_SECURE_COOKIES").ok().as_deref(),
            config.secure_cookies.as_deref(),
        ),
        allow_signup: parse_bool_setting(
            env::var("ROBOMINER_ALLOW_SIGNUP").ok().as_deref(),
            config.allow_signup.as_deref(),
        ),
        trust_proxy: parse_bool_setting(
            env::var("ROBOMINER_TRUST_PROXY").ok().as_deref(),
            config.trust_proxy.as_deref(),
        ),
        allow_insecure_dev_secret: parse_bool_setting(
            env::var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET")
                .ok()
                .as_deref(),
            config.allow_insecure_dev_secret.as_deref(),
        ),
    }
}

pub(crate) fn parse_bool_setting(env_value: Option<&str>, config_value: Option<&str>) -> bool {
    parse_optional_bool_setting(env_value, config_value).unwrap_or(false)
}

pub(crate) fn parse_optional_bool_setting(
    env_value: Option<&str>,
    config_value: Option<&str>,
) -> Option<bool> {
    if let Some(value) = env_value {
        return Some(matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        ));
    }

    config_value.map(|value| {
        matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_setting_defaults_to_false_when_unset() {
        assert!(!parse_bool_setting(None, None));
    }

    #[test]
    fn parse_optional_bool_setting_is_none_when_unset() {
        assert_eq!(parse_optional_bool_setting(None, None), None);
    }

    #[test]
    fn parse_bool_setting_accepts_truthy_spellings() {
        for value in ["1", "true", "TRUE", "yes", "YES", "on", "ON", " 1 "] {
            assert!(
                parse_bool_setting(Some(value), None),
                "env {value:?} should be true"
            );
            assert!(
                parse_bool_setting(None, Some(value)),
                "config {value:?} should be true"
            );
        }
    }

    #[test]
    fn parse_bool_setting_rejects_falsey_spellings() {
        for value in ["0", "false", "no", "off", "", "maybe"] {
            assert!(
                !parse_bool_setting(Some(value), Some("1")),
                "env {value:?} should win as false"
            );
            assert!(
                !parse_bool_setting(None, Some(value)),
                "config {value:?} should be false"
            );
        }
    }

    #[test]
    fn parse_bool_setting_prefers_env_over_config() {
        assert!(parse_bool_setting(Some("1"), Some("0")));
        assert!(!parse_bool_setting(Some("0"), Some("1")));
    }

    #[test]
    fn web_settings_reads_config_when_matching_env_vars_are_unset() {
        let config = LegacyAppConfig {
            host: Some("10.0.0.2".to_string()),
            port: Some("9090".to_string()),
            web_root: Some("/opt/static".to_string()),
            session_secret: Some("secret".to_string()),
            secure_cookies: Some("1".to_string()),
            allow_signup: Some("1".to_string()),
            trust_proxy: Some("true".to_string()),
            ..LegacyAppConfig::default()
        };

        let settings = web_settings(&config, Path::new("/default/static"));

        if env::var("HOST").is_err() {
            assert_eq!(settings.host, "10.0.0.2");
        }
        if env::var("PORT").is_err() {
            assert_eq!(settings.port, "9090");
        }
        if env::var("ROBOMINER_WEB_ROOT").is_err() {
            assert_eq!(settings.static_root, PathBuf::from("/opt/static"));
        }
        if env::var("ROBOMINER_SESSION_SECRET").is_err() {
            assert_eq!(settings.session_secret.as_deref(), Some("secret"));
        }
        if env::var("ROBOMINER_SECURE_COOKIES").is_err() {
            assert_eq!(settings.secure_cookies, Some(true));
        }
        if env::var("ROBOMINER_ALLOW_SIGNUP").is_err() {
            assert!(settings.allow_signup);
        }
        if env::var("ROBOMINER_TRUST_PROXY").is_err() {
            assert!(settings.trust_proxy);
        }
    }

    #[test]
    fn web_settings_defaults_when_config_and_env_are_empty() {
        let settings = web_settings(&LegacyAppConfig::default(), Path::new("/default/static"));
        if env::var("HOST").is_err() {
            assert_eq!(settings.host, "127.0.0.1");
        }
        if env::var("PORT").is_err() {
            assert_eq!(settings.port, "8080");
        }
        if env::var("ROBOMINER_WEB_ROOT").is_err() {
            assert_eq!(settings.static_root, PathBuf::from("/default/static"));
        }
        if env::var("ROBOMINER_SECURE_COOKIES").is_err() {
            assert_eq!(settings.secure_cookies, None);
        }
        if env::var("ROBOMINER_ALLOW_SIGNUP").is_err() {
            assert!(!settings.allow_signup);
        }
        if env::var("ROBOMINER_TRUST_PROXY").is_err() {
            assert!(!settings.trust_proxy);
        }
        if env::var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET").is_err() {
            assert!(!settings.allow_insecure_dev_secret);
        }
    }

    #[test]
    fn web_settings_accepts_insecure_dev_secret_from_typed_config() {
        let config = LegacyAppConfig {
            allow_insecure_dev_secret: Some("1".to_string()),
            ..LegacyAppConfig::default()
        };
        let settings = web_settings(&config, Path::new("/default/static"));
        if env::var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET").is_err() {
            assert!(settings.allow_insecure_dev_secret);
        }
    }
}
