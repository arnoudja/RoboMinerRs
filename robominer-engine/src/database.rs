use anyhow::{Context, Result, anyhow};

pub(crate) async fn connect_database(
    database_url: Option<String>,
) -> Result<robominer_db::MySqlPool> {
    robominer_db::connect_from_cli(database_url)
        .await
        .map_err(|error| anyhow!(error))
        .context("failed to connect to database")
}
