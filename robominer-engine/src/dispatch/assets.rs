use std::path::PathBuf;

use anyhow::Result;

use super::ensure_positive_user_id;
use crate::assets::user_ore_asset_states;
use crate::cli::AssetsCommand;
use crate::database::connect_database;

pub(crate) async fn dispatch_assets(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: AssetsCommand,
) -> Result<()> {
    match command {
        AssetsCommand::OreStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            user_ore_asset_states(&pool, user_id).await
        }
    }
}
