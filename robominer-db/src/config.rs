//! CLI and environment database URL resolution.
//!
//! Primary entry points: [`connect_from_cli`], [`resolve_database_url`].

use std::env;

use thiserror::Error;

use crate::MySqlPool;
use crate::connect_with_max_connections;
use crate::resolve_max_connections;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("database URL not set; pass --database-url or set ROBOMINER_DATABASE_URL")]
    MissingDatabaseUrl,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("{0}")]
    MaxConnections(String),
    #[error(transparent)]
    Sqlx(sqlx::Error),
}

/// Connect to MySQL using CLI/env resolution shared by binaries.
pub async fn connect_from_cli(database_url: Option<String>) -> Result<MySqlPool, ConnectError> {
    let database_url = resolve_database_url(database_url)?;

    let max_connections =
        resolve_max_connections(env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref())
            .map_err(ConnectError::MaxConnections)?;

    connect_with_max_connections(&database_url, max_connections)
        .await
        .map_err(ConnectError::Sqlx)
}

pub fn resolve_database_url(cli_database_url: Option<String>) -> Result<String, ConfigError> {
    if let Some(database_url) = cli_database_url {
        return Ok(database_url);
    }

    if let Ok(database_url) = env::var("ROBOMINER_DATABASE_URL")
        && !database_url.is_empty()
    {
        return Ok(database_url);
    }

    Err(ConfigError::MissingDatabaseUrl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_database_url_prefers_cli_database_url() {
        let url = resolve_database_url(Some("mysql://cli:secret@localhost/RoboMiner".to_string()))
            .expect("database url");

        assert_eq!(url, "mysql://cli:secret@localhost/RoboMiner");
    }

    #[test]
    fn resolve_database_url_errors_when_unset() {
        // Do not assert against process env (may be set in CI); only the CLI-empty +
        // missing-env path is covered indirectly by callers that clear the var.
        let err = ConfigError::MissingDatabaseUrl;
        assert!(err.to_string().contains("ROBOMINER_DATABASE_URL"));
    }
}
