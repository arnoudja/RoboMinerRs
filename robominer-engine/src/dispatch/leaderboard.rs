use anyhow::{Result, ensure};

use crate::cli::LeaderboardCommand;
use crate::database::connect_database;
use crate::leaderboard::leaderboard_states;

pub(crate) async fn dispatch_leaderboard(
    database_url: Option<String>,
    command: LeaderboardCommand,
) -> Result<()> {
    match command {
        LeaderboardCommand::States { max_entries } => {
            ensure!(max_entries > 0, "--max-entries must be greater than zero");
            let pool = connect_database(database_url).await?;
            leaderboard_states(&pool, max_entries).await
        }
    }
}
