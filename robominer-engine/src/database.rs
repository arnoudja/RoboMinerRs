use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;

pub(crate) async fn connect_database(
    database_url: Option<String>,
    config: Option<PathBuf>,
) -> Result<robominer_db::MySqlPool> {
    let database_url = robominer_db::resolve_database_url(database_url, config, "robominer-engine")
        .map_err(|error| anyhow!(error))?;

    robominer_db::connect(&database_url)
        .await
        .context("failed to connect to database")
}
