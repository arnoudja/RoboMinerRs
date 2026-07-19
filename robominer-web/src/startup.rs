use std::collections::HashMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

use crate::{ServerConfig, block_on_database, web_settings};

/// Resolve legacy config for the web process. Missing files are non-fatal.
pub fn load_legacy_config(
    config: Option<PathBuf>,
) -> io::Result<(PathBuf, HashMap<String, String>)> {
    match robominer_db::load_legacy_config(config, "robominer-web") {
        Ok((config_path, config)) => Ok((config_path, config)),
        Err(robominer_db::ConfigError::MissingConfigFile) => {
            Ok((PathBuf::from("<none>"), HashMap::new()))
        }
        Err(error) => Err(io::Error::other(format!("failed to load config: {error}"))),
    }
}

/// Connect to MySQL when a database URL is configured; otherwise return `None`.
pub fn connect_database(
    database_url: Option<String>,
    config: Option<PathBuf>,
    legacy_config: &HashMap<String, String>,
) -> io::Result<Option<robominer_db::MySqlPool>> {
    let database_url =
        match robominer_db::resolve_database_url(database_url, config, "robominer-web") {
            Ok(url) => url,
            Err(robominer_db::ConfigError::MissingConfigFile) => return Ok(None),
            Err(error) => return Err(io::Error::other(error.to_string())),
        };

    let max_connections = robominer_db::resolve_max_connections(
        env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref(),
        robominer_db::config_value(legacy_config, "dbmaxconnections"),
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
    legacy_config: &HashMap<String, String>,
    database_pool: Option<robominer_db::MySqlPool>,
) -> io::Result<(String, u16, ServerConfig)> {
    let settings = web_settings(legacy_config, &default_web_root());
    let session_secret =
        crate::resolve_session_secret(settings.session_secret.as_deref(), &settings.host)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let session_ttl_secs = crate::resolve_session_ttl_secs(
        settings.session_ttl_secs.as_deref(),
        settings.session_ttl_hours.as_deref(),
        robominer_db::config_value(legacy_config, "sessionttlsecs"),
        robominer_db::config_value(legacy_config, "sessionttlhours"),
    )
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    crate::configure_session_secret(&session_secret);
    crate::configure_secure_cookies(settings.secure_cookies);
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
    fn load_legacy_config_treats_missing_file_as_empty() {
        let missing = PathBuf::from("/tmp/robominer-web-missing-config-does-not-exist.conf");
        let (path, config) =
            load_legacy_config(Some(missing)).expect("missing config is non-fatal");
        assert_eq!(path, PathBuf::from("<none>"));
        assert!(config.is_empty());
    }

    #[test]
    fn load_legacy_config_reads_temp_file() {
        let path =
            std::env::temp_dir().join(format!("robominer-web-startup-{}.conf", std::process::id()));
        std::fs::write(&path, "host 127.0.0.1\nport 18080\n").expect("write config");
        let (loaded_path, config) =
            load_legacy_config(Some(path.clone())).expect("temp config should load");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded_path, path);
        assert_eq!(config.get("host").map(String::as_str), Some("127.0.0.1"));
        assert_eq!(config.get("port").map(String::as_str), Some("18080"));
    }

    #[test]
    fn connect_database_returns_none_without_url_or_config() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = env::var("ROBOMINER_DATABASE_URL").ok();
        unsafe {
            env::remove_var("ROBOMINER_DATABASE_URL");
        }
        let missing = PathBuf::from("/tmp/robominer-web-missing-db-config.conf");
        let result = connect_database(None, Some(missing), &HashMap::new());
        match previous {
            Some(value) => unsafe {
                env::set_var("ROBOMINER_DATABASE_URL", value);
            },
            None => unsafe {
                env::remove_var("ROBOMINER_DATABASE_URL");
            },
        }
        let pool = result.expect("missing config should not be an IO error");
        assert!(pool.is_none());
    }

    #[test]
    fn prepare_server_config_uses_localhost_defaults() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Clear signup/proxy overrides so defaults are deterministic.
        let previous_signup = env::var("ROBOMINER_ALLOW_SIGNUP").ok();
        let previous_proxy = env::var("ROBOMINER_TRUST_PROXY").ok();
        unsafe {
            env::remove_var("ROBOMINER_ALLOW_SIGNUP");
            env::remove_var("ROBOMINER_TRUST_PROXY");
        }

        let (host, port, config) =
            prepare_server_config(&HashMap::new(), None).expect("defaults should prepare");

        match previous_signup {
            Some(value) => unsafe {
                env::set_var("ROBOMINER_ALLOW_SIGNUP", value);
            },
            None => unsafe {
                env::remove_var("ROBOMINER_ALLOW_SIGNUP");
            },
        }
        match previous_proxy {
            Some(value) => unsafe {
                env::set_var("ROBOMINER_TRUST_PROXY", value);
            },
            None => unsafe {
                env::remove_var("ROBOMINER_TRUST_PROXY");
            },
        }

        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8080);
        assert!(!config.allow_signup);
        assert!(!config.trust_proxy);
        assert!(config.database_pool.is_none());
        assert!(config.static_root.ends_with("static"));
    }
}
