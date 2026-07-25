use std::path::PathBuf;

use anyhow::Result;

use crate::cli::MigrateCommand;
use crate::database::connect_database;
use crate::migrate::{migrate, migrate_status};

pub(crate) async fn dispatch_migrate(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: MigrateCommand,
) -> Result<()> {
    match command {
        MigrateCommand::Apply => {
            let pool = connect_database(database_url, config).await?;
            migrate(&pool).await
        }
        MigrateCommand::Status { check } => {
            let pool = connect_database(database_url, config).await?;
            migrate_status(&pool, check).await
        }
    }
}
