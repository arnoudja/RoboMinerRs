use std::env;
use std::path::{Path, PathBuf};

pub struct WebSettings {
    pub host: String,
    pub port: String,
    pub static_root: PathBuf,
    pub session_secret: Option<String>,
    pub session_ttl_secs: Option<String>,
    pub session_ttl_hours: Option<String>,
    /// `None` means unset → Secure cookies stay off (set `ROBOMINER_SECURE_COOKIES=1` for HTTPS).
    pub secure_cookies: Option<bool>,
    pub allow_signup: bool,
    pub trust_proxy: bool,
    pub allow_insecure_dev_secret: bool,
}

pub fn web_settings(default_static_root: &Path) -> WebSettings {
    WebSettings {
        host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        static_root: env::var("ROBOMINER_WEB_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_static_root.to_path_buf()),
        session_secret: env::var("ROBOMINER_SESSION_SECRET").ok(),
        session_ttl_secs: env::var("ROBOMINER_SESSION_TTL_SECS").ok(),
        session_ttl_hours: env::var("ROBOMINER_SESSION_TTL_HOURS").ok(),
        secure_cookies: parse_optional_bool_setting(
            env::var("ROBOMINER_SECURE_COOKIES").ok().as_deref(),
        ),
        allow_signup: parse_bool_setting(env::var("ROBOMINER_ALLOW_SIGNUP").ok().as_deref()),
        trust_proxy: parse_bool_setting(env::var("ROBOMINER_TRUST_PROXY").ok().as_deref()),
        allow_insecure_dev_secret: parse_bool_setting(
            env::var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET")
                .ok()
                .as_deref(),
        ),
    }
}

pub(crate) fn parse_bool_setting(env_value: Option<&str>) -> bool {
    parse_optional_bool_setting(env_value).unwrap_or(false)
}

pub(crate) fn parse_optional_bool_setting(env_value: Option<&str>) -> Option<bool> {
    env_value.map(|value| {
        matches!(
            value.trim(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_cleared_web_env<T>(f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let keys = [
            "HOST",
            "PORT",
            "ROBOMINER_WEB_ROOT",
            "ROBOMINER_SESSION_SECRET",
            "ROBOMINER_SESSION_TTL_SECS",
            "ROBOMINER_SESSION_TTL_HOURS",
            "ROBOMINER_SECURE_COOKIES",
            "ROBOMINER_ALLOW_SIGNUP",
            "ROBOMINER_TRUST_PROXY",
            "ROBOMINER_ALLOW_INSECURE_DEV_SECRET",
        ];
        let previous: Vec<_> = keys.iter().map(|key| (*key, env::var(key).ok())).collect();
        for key in &keys {
            unsafe {
                env::remove_var(key);
            }
        }
        let result = f();
        for (key, value) in previous {
            match value {
                Some(value) => unsafe {
                    env::set_var(key, value);
                },
                None => unsafe {
                    env::remove_var(key);
                },
            }
        }
        result
    }

    #[test]
    fn parse_bool_setting_defaults_to_false_when_unset() {
        assert!(!parse_bool_setting(None));
    }

    #[test]
    fn parse_optional_bool_setting_is_none_when_unset() {
        assert_eq!(parse_optional_bool_setting(None), None);
    }

    #[test]
    fn parse_bool_setting_accepts_truthy_spellings() {
        for value in ["1", "true", "TRUE", "yes", "YES", "on", "ON", " 1 "] {
            assert!(
                parse_bool_setting(Some(value)),
                "env {value:?} should be true"
            );
        }
    }

    #[test]
    fn parse_bool_setting_rejects_falsey_spellings() {
        for value in ["0", "false", "no", "off", "", "maybe"] {
            assert!(
                !parse_bool_setting(Some(value)),
                "env {value:?} should be false"
            );
        }
    }

    #[test]
    fn web_settings_defaults_when_env_is_empty() {
        with_cleared_web_env(|| {
            let settings = web_settings(Path::new("/default/static"));
            assert_eq!(settings.host, "127.0.0.1");
            assert_eq!(settings.port, "8080");
            assert_eq!(settings.static_root, PathBuf::from("/default/static"));
            assert_eq!(settings.secure_cookies, None);
            assert!(!settings.allow_signup);
            assert!(!settings.trust_proxy);
            assert!(!settings.allow_insecure_dev_secret);
        });
    }

    #[test]
    fn web_settings_reads_env_overrides() {
        with_cleared_web_env(|| {
            unsafe {
                env::set_var("HOST", "10.0.0.2");
                env::set_var("PORT", "9090");
                env::set_var("ROBOMINER_WEB_ROOT", "/opt/static");
                env::set_var("ROBOMINER_SESSION_SECRET", "secret");
                env::set_var("ROBOMINER_SECURE_COOKIES", "1");
                env::set_var("ROBOMINER_ALLOW_SIGNUP", "1");
                env::set_var("ROBOMINER_TRUST_PROXY", "true");
                env::set_var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET", "1");
            }
            let settings = web_settings(Path::new("/default/static"));
            assert_eq!(settings.host, "10.0.0.2");
            assert_eq!(settings.port, "9090");
            assert_eq!(settings.static_root, PathBuf::from("/opt/static"));
            assert_eq!(settings.session_secret.as_deref(), Some("secret"));
            assert_eq!(settings.secure_cookies, Some(true));
            assert!(settings.allow_signup);
            assert!(settings.trust_proxy);
            assert!(settings.allow_insecure_dev_secret);
        });
    }
}
