use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const SHARED_CONFIG_PATH: &str = "/etc/robominer/robominer.conf";
pub const LEGACY_ENGINE_CONFIG_PATH: &str = "/etc/robominer/robominer-engine.conf";

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

pub fn read_legacy_config(config_path: &Path) -> Result<HashMap<String, String>, ConfigError> {
    let contents = fs::read_to_string(config_path)?;
    Ok(parse_legacy_config(&contents))
}

pub fn parse_legacy_config(contents: &str) -> HashMap<String, String> {
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

pub fn database_url_from_config(config: &HashMap<String, String>) -> Result<String, ConfigError> {
    let server = required_config_value(config, "dbserver")?;
    let user = required_config_value(config, "dbuser")?;
    let password = required_config_value(config, "dbpassword")?;
    let database = required_config_value(config, "dbdatabase")?;

    Ok(format!("mysql://{user}:{password}@{server}/{database}"))
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

    database_url_from_config(&config)
}

pub fn load_legacy_config(
    cli_config: Option<PathBuf>,
    executable_stem: &str,
) -> Result<(PathBuf, HashMap<String, String>), ConfigError> {
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

pub fn config_value<'a>(config: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    config.get(&key.to_ascii_lowercase()).map(String::as_str)
}

fn required_config_value(
    config: &HashMap<String, String>,
    key: &str,
) -> Result<String, ConfigError> {
    let normalized_key = key.to_ascii_lowercase();
    match config.get(&normalized_key) {
        Some(value) if !value.is_empty() => Ok(value.clone()),
        Some(_) => Err(ConfigError::EmptyKey(normalized_key)),
        None => Err(ConfigError::MissingKey(normalized_key)),
    }
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
        let config = parse_legacy_config(
            "# database\n\
             dbserver localhost\n\
             DBUSER robominer\n\
             dbpassword secret\n\
             dbdatabase RoboMiner\n",
        );

        assert_eq!(config.get("dbserver"), Some(&"localhost".to_string()));
        assert_eq!(config.get("dbuser"), Some(&"robominer".to_string()));
        assert_eq!(
            database_url_from_config(&config).expect("database url"),
            "mysql://robominer:secret@localhost/RoboMiner"
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
             dbdatabase RoboMiner\n",
        )
        .expect("write config");

        let config = read_legacy_config(&config_path).expect("read config");
        assert_eq!(
            database_url_from_config(&config).expect("database url"),
            "mysql://user:pass@db.example/RoboMiner"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
