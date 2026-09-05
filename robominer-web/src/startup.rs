use std::env;
use std::io;
use std::path::{Path, PathBuf};

use crate::{ServerConfig, block_on_database, web_settings};

/// Connect to MySQL when a database URL is configured; otherwise return `None`.
pub fn connect_database(
    database_url: Option<String>,
) -> io::Result<Option<robominer_db::MySqlPool>> {
    let database_url = match robominer_db::resolve_database_url(database_url) {
        Ok(url) => url,
        Err(robominer_db::ConfigError::MissingDatabaseUrl) => return Ok(None),
    };

    let max_connections = robominer_db::resolve_max_connections(
        env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref(),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    let pool = block_on_database(robominer_db::connect_with_max_connections(
        &database_url,
        max_connections,
    ))
    .map_err(|error| io::Error::other(format!("failed to connect to database: {error}")))?;

    Ok(Some(pool))
}

pub fn default_web_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("static")
}

/// Apply session settings and build the Axum `ServerConfig` (without binding a listener).
pub fn prepare_server_config(
    database_pool: Option<robominer_db::MySqlPool>,
) -> io::Result<(String, u16, ServerConfig)> {
    let settings = web_settings(&default_web_root());
    crate::validate_trust_proxy_bind(&settings.host, settings.trust_proxy)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let session_secret = crate::resolve_session_secret(
        settings.session_secret.as_deref(),
        &settings.host,
        settings.allow_insecure_dev_secret,
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let session_ttl_secs = crate::resolve_session_ttl_secs(
        settings.session_ttl_secs.as_deref(),
        settings.session_ttl_hours.as_deref(),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    crate::configure_session_secret(&session_secret)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let secure_cookies = crate::resolve_secure_cookies(
        settings.secure_cookies,
        &settings.host,
        settings.trust_proxy,
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    crate::configure_secure_cookies(secure_cookies);
    crate::configure_session_ttl_secs(session_ttl_secs);

    let port = settings
        .port
        .parse::<u16>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;

    Ok((
        settings.host,
        port,
        ServerConfig {
            static_root: settings.static_root,
            database_pool,
            allow_signup: settings.allow_signup,
            trust_proxy: settings.trust_proxy,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_web_root_points_at_crate_static_dir() {
        let root = default_web_root();
        assert!(root.ends_with("static"));
        assert!(root.is_dir(), "expected {} to exist", root.display());
    }

    #[test]
    fn connect_database_returns_none_without_url() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = env::var("ROBOMINER_DATABASE_URL").ok();
        unsafe {
            env::remove_var("ROBOMINER_DATABASE_URL");
        }
        let result = connect_database(None);
        match previous {
            Some(value) => unsafe {
                env::set_var("ROBOMINER_DATABASE_URL", value);
            },
            None => unsafe {
                env::remove_var("ROBOMINER_DATABASE_URL");
            },
        }
        let pool = result.expect("missing url should not be an IO error");
        assert!(pool.is_none());
    }

    #[test]
    fn prepare_server_config_uses_localhost_defaults() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let keys = [
            "HOST",
            "PORT",
            "ROBOMINER_ALLOW_SIGNUP",
            "ROBOMINER_TRUST_PROXY",
            "ROBOMINER_SESSION_SECRET",
            "ROBOMINER_ALLOW_INSECURE_DEV_SECRET",
        ];
        let previous: Vec<_> = keys.iter().map(|key| (*key, env::var(key).ok())).collect();
        unsafe {
            env::remove_var("HOST");
            env::remove_var("PORT");
            env::remove_var("ROBOMINER_ALLOW_SIGNUP");
            env::remove_var("ROBOMINER_TRUST_PROXY");
            env::remove_var("ROBOMINER_SESSION_SECRET");
            env::set_var("ROBOMINER_ALLOW_INSECURE_DEV_SECRET", "1");
        }

        let (host, port, config) = prepare_server_config(None).expect("defaults should prepare");

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

        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080);
        assert!(!config.allow_signup);
        assert!(!config.trust_proxy);
        assert!(config.database_pool.is_none());
        assert!(config.static_root.ends_with("static"));
    }
}
