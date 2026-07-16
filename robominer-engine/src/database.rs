use anyhow::{Context, Result, anyhow};
use std::env;
use std::path::PathBuf;

pub(crate) async fn connect_database(
    database_url: Option<String>,
    config: Option<PathBuf>,
) -> Result<robominer_db::MySqlPool> {
    let database_url =
        robominer_db::resolve_database_url(database_url, config.clone(), "robominer-engine")
            .map_err(|error| anyhow!(error))?;

    let config_value = match robominer_db::load_legacy_config(config, "robominer-engine") {
        Ok((_, config_map)) => {
            robominer_db::config_value(&config_map, "dbmaxconnections").map(str::to_owned)
        }
        Err(robominer_db::ConfigError::MissingConfigFile) => None,
        Err(error) => return Err(anyhow!(error)),
    };
    let max_connections = robominer_db::resolve_max_connections(
        env::var("ROBOMINER_DB_MAX_CONNECTIONS").ok().as_deref(),
        config_value.as_deref(),
    )
    .map_err(|error| anyhow!(error))?;

    robominer_db::connect_with_max_connections(&database_url, max_connections)
        .await
        .context("failed to connect to database")
}
