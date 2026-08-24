use anyhow::{Context, Result, anyhow};

pub(crate) async fn connect_database(
    database_url: Option<String>,
    config: Option<std::path::PathBuf>,
) -> Result<robominer_db::MySqlPool> {
    robominer_db::connect_from_cli(database_url, config, "robominer-engine")
        .await
        .map_err(|error| anyhow!(error))
        .context("failed to connect to database")
}
