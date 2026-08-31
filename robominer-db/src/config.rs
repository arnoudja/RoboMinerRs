use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::MySqlPool;
use crate::connect_with_max_connections;
use crate::resolve_max_connections;

pub const SHARED_CONFIG_PATH: &str = "/etc/robominer/robominer.conf";
pub const LEGACY_ENGINE_CONFIG_PATH: &str = "/etc/robominer/robominer-engine.conf";

/// Known keys in the legacy key=value config file (case-insensitive on disk).
mod keys {
    pub const DB_SERVER: &str = "dbserver";
    pub const DB_USER: &str = "dbuser";
    pub const DB_PASSWORD: &str = "dbpassword";
    pub const DB_DATABASE: &str = "dbdatabase";
    pub const DB_MAX_CONNECTIONS: &str = "dbmaxconnections";
    pub const HOST: &str = "host";
    pub const PORT: &str = "port";
    pub const WEB_ROOT: &str = "webroot";
    pub const SESSION_SECRET: &str = "sessionsecret";
    pub const SESSION_TTL_SECS: &str = "sessionttlsecs";
    pub const SESSION_TTL_HOURS: &str = "sessionttlhours";
    pub const SECURE_COOKIES: &str = "securecookies";
    pub const ALLOW_SIGNUP: &str = "allowsignup";
    pub const TRUST_PROXY: &str = "trustproxy";
    pub const ALLOW_INSECURE_DEV_SECRET_ALIASES: &[&str] = &[
        "allowinsecuredevsecret",
        "allow-insecure-dev-secret",
        "insecure-dev-secret",
        "insecuredevsecret",
    ];
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    MissingConfigFile,
    MissingKey(String),
    EmptyKey(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::MissingConfigFile => write!(f, "no RoboMiner config file found"),
            Self::MissingKey(key) => write!(f, "config key {key} is required"),
            Self::EmptyKey(key) => write!(f, "config key {key} must not be empty"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::MissingConfigFile | Self::MissingKey(_) | Self::EmptyKey(_) => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub enum ConnectError {
    Config(ConfigError),
    MaxConnections(String),
    Sqlx(sqlx::Error),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::MaxConnections(message) => write!(f, "{message}"),
            Self::Sqlx(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ConnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::MaxConnections(_) => None,
            Self::Sqlx(error) => Some(error),
        }
    }
}

/// Typed view of a legacy RoboMiner key=value config file.
///
/// File parsing still goes through a key/value map (legacy format), but call sites
/// read named fields instead of magic strings.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LegacyAppConfig {
    pub db_server: Option<String>,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_database: Option<String>,
    pub db_max_connections: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub web_root: Option<String>,
    pub session_secret: Option<String>,
    pub session_ttl_secs: Option<String>,
    pub session_ttl_hours: Option<String>,
    pub secure_cookies: Option<String>,
    pub allow_signup: Option<String>,
    pub trust_proxy: Option<String>,
    pub allow_insecure_dev_secret: Option<String>,
}

impl LegacyAppConfig {
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        Self {
            db_server: map_get(map, keys::DB_SERVER),
            db_user: map_get(map, keys::DB_USER),
            db_password: map_get(map, keys::DB_PASSWORD),
            db_database: map_get(map, keys::DB_DATABASE),
            db_max_connections: map_get(map, keys::DB_MAX_CONNECTIONS),
            host: map_get(map, keys::HOST),
            port: map_get(map, keys::PORT),
            web_root: map_get(map, keys::WEB_ROOT),
            session_secret: map_get(map, keys::SESSION_SECRET),
            session_ttl_secs: map_get(map, keys::SESSION_TTL_SECS),
            session_ttl_hours: map_get(map, keys::SESSION_TTL_HOURS),
            secure_cookies: map_get(map, keys::SECURE_COOKIES),
            allow_signup: map_get(map, keys::ALLOW_SIGNUP),
            trust_proxy: map_get(map, keys::TRUST_PROXY),
            allow_insecure_dev_secret: keys::ALLOW_INSECURE_DEV_SECRET_ALIASES
                .iter()
                .find_map(|key| map_get(map, key)),
        }
    }

    pub fn parse(contents: &str) -> Self {
        Self::from_map(&parse_legacy_config_map(contents))
    }

    pub fn database_url(&self) -> Result<String, ConfigError> {
        let server = required_field(&self.db_server, keys::DB_SERVER)?;
        let user = required_field(&self.db_user, keys::DB_USER)?;
        let password = required_field(&self.db_password, keys::DB_PASSWORD)?;
        let database = required_field(&self.db_database, keys::DB_DATABASE)?;

        Ok(format!(
            "mysql://{}:{}@{}/{}",
            percent_encode_userinfo(user),
            percent_encode_userinfo(password),
            server,
            database
        ))
    }
}

fn map_get(map: &HashMap<String, String>, key: &str) -> Option<String> {
    map.get(&key.to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .cloned()
}

fn required_field<'a>(value: &'a Option<String>, key: &str) -> Result<&'a str, ConfigError> {
    match value {
        Some(value) if !value.is_empty() => Ok(value.as_str()),
        Some(_) => Err(ConfigError::EmptyKey(key.to_ascii_lowercase())),
        None => Err(ConfigError::MissingKey(key.to_ascii_lowercase())),
    }
}

/// Connect to MySQL using CLI/env/config resolution shared by binaries.
pub async fn connect_from_cli(
    database_url: Option<String>,
    config: Option<PathBuf>,
    executable_stem: &str,
) -> Result<MySqlPool, ConnectError> {
    let database_url = resolve_database_url(database_url, config.clone(), executable_stem)
        .map_err(ConnectError::Config)?;

    let config_value = match load_legacy_config(config, executable_stem) {
        Ok((_, app_config)) => app_config.db_max_connections.clone(),
        Err(ConfigError::MissingConfigFile) => None,
        Err(error) => return Err(ConnectError::Config(error)),
    };
    let max_connections = resolve_max_connections(
        env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref(),
        config_value.as_deref(),
    )
    .map_err(ConnectError::MaxConnections)?;

    connect_with_max_connections(&database_url, max_connections)
        .await
        .map_err(ConnectError::Sqlx)
}

pub fn read_legacy_config(config_path: &Path) -> Result<LegacyAppConfig, ConfigError> {
    let contents = fs::read_to_string(config_path)?;
    Ok(LegacyAppConfig::parse(&contents))
}

/// Parse a legacy key=value config file into a normalized lowercase-key map.
/// Prefer [`LegacyAppConfig::parse`] / [`read_legacy_config`] at call sites.
pub fn parse_legacy_config(contents: &str) -> HashMap<String, String> {
    parse_legacy_config_map(contents)
}

fn parse_legacy_config_map(contents: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let Some(key) = parts.next() else {
            continue;
        };

        let value = parts.next().unwrap_or("").trim();
        result.insert(key.to_ascii_lowercase(), value.to_owned());
    }

    result
}

pub fn database_url_from_config(config: &LegacyAppConfig) -> Result<String, ConfigError> {
    config.database_url()
}

/// Encode userinfo components so passwords with `@`, `:`, `/`, etc. stay valid in URLs.
fn percent_encode_userinfo(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
                encoded.push(char::from(b"0123456789ABCDEF"[(byte & 0xf) as usize]));
            }
        }
    }
    encoded
}

pub fn resolve_database_url(
    cli_database_url: Option<String>,
    cli_config: Option<PathBuf>,
    executable_stem: &str,
) -> Result<String, ConfigError> {
    if let Some(database_url) = cli_database_url {
        return Ok(database_url);
    }

    if let Ok(database_url) = env::var("ROBOMINER_DATABASE_URL")
        && !database_url.is_empty()
    {
        return Ok(database_url);
    }

    let config_path = match cli_config {
        Some(config_path) => config_path,
        None => locate_config_file(executable_stem)?,
    };

    let config = read_legacy_config(&config_path).map_err(|error| match error {
        ConfigError::Io(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            ConfigError::MissingConfigFile
        }
        other => other,
    })?;

    config.database_url()
}

pub fn load_legacy_config(
    cli_config: Option<PathBuf>,
    executable_stem: &str,
) -> Result<(PathBuf, LegacyAppConfig), ConfigError> {
    let config_path = match cli_config {
        Some(config_path) => config_path,
        None => locate_config_file(executable_stem)?,
    };

    let config = read_legacy_config(&config_path).map_err(|error| match error {
        ConfigError::Io(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            ConfigError::MissingConfigFile
        }
        other => other,
    })?;

    Ok((config_path, config))
}

/// Look up a raw key in a legacy map. Prefer [`LegacyAppConfig`] fields at new call sites.
pub fn config_value<'a>(config: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    config.get(&key.to_ascii_lowercase()).map(String::as_str)
}

fn locate_config_file(executable_stem: &str) -> Result<PathBuf, ConfigError> {
    for candidate in config_search_paths(executable_stem) {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(ConfigError::MissingConfigFile)
}

fn config_search_paths(executable_stem: &str) -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from(SHARED_CONFIG_PATH)];

    if executable_stem == "robominer-engine" {
        paths.push(PathBuf::from(LEGACY_ENGINE_CONFIG_PATH));
    }

    if let Ok(mut executable_path) = env::current_exe() {
        executable_path.set_file_name(format!("{executable_stem}.conf"));
        paths.push(executable_path);
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_config_normalizes_keys_and_ignores_comments() {
        let config = LegacyAppConfig::parse(
            "# database\n\
             dbserver localhost\n\
             DBUSER robominer\n\
             dbpassword secret\n\
             dbdatabase RoboMiner\n",
        );

        assert_eq!(config.db_server.as_deref(), Some("localhost"));
        assert_eq!(config.db_user.as_deref(), Some("robominer"));
        assert_eq!(
            config.database_url().expect("database url"),
            "mysql://robominer:secret@localhost/RoboMiner"
        );
    }

    #[test]
    fn database_url_from_config_percent_encodes_password_specials() {
        let config = LegacyAppConfig::parse(
            "dbserver localhost\n\
             dbuser robominer\n\
             dbpassword p@ss:w/ord\n\
             dbdatabase RoboMiner\n",
        );

        assert_eq!(
            config.database_url().expect("database url"),
            "mysql://robominer:p%40ss%3Aw%2Ford@localhost/RoboMiner"
        );
    }

    #[test]
    fn resolve_database_url_prefers_cli_database_url() {
        let url = resolve_database_url(
            Some("mysql://cli:secret@localhost/RoboMiner".to_string()),
            None,
            "robominer-web",
        )
        .expect("database url");

        assert_eq!(url, "mysql://cli:secret@localhost/RoboMiner");
    }

    #[test]
    fn read_legacy_config_from_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("robominer-config-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let config_path = temp_dir.join("robominer.conf");
        fs::write(
            &config_path,
            "dbserver db.example\n\
             dbuser user\n\
             dbpassword pass\n\
             dbdatabase RoboMiner\n\
             host 10.0.0.2\n\
             allowsignup 1\n",
        )
        .expect("write config");

        let config = read_legacy_config(&config_path).expect("read config");
        assert_eq!(
            config.database_url().expect("database url"),
            "mysql://user:pass@db.example/RoboMiner"
        );
        assert_eq!(config.host.as_deref(), Some("10.0.0.2"));
        assert_eq!(config.allow_signup.as_deref(), Some("1"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn legacy_app_config_reads_insecure_dev_secret_aliases() {
        for key in keys::ALLOW_INSECURE_DEV_SECRET_ALIASES {
            let config = LegacyAppConfig::parse(&format!("{key} 1\n"));
            assert_eq!(
                config.allow_insecure_dev_secret.as_deref(),
                Some("1"),
                "alias {key}"
            );
        }
    }
}
