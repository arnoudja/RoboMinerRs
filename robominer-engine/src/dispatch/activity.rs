use anyhow::{Result, ensure};

use super::ensure_positive_user_id;
use crate::activity::{activity_states, rally_view_state};
use crate::cli::ActivityCommand;
use crate::database::connect_database;

pub(crate) async fn dispatch_activity(
    database_url: Option<String>,
    command: ActivityCommand,
) -> Result<()> {
    match command {
        ActivityCommand::States {
            user_id,
            max_users,
            max_rallies,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(max_users > 0, "--max-users must be greater than zero");
            ensure!(max_rallies > 0, "--max-rallies must be greater than zero");
            let pool = connect_database(database_url).await?;
            activity_states(&pool, max_users, max_rallies).await
        }
        ActivityCommand::RallyViewState {
            user_id,
            rally_result_id,
            require_user_result,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                rally_result_id > 0,
                "--rally-result-id must be greater than zero"
            );
            let pool = connect_database(database_url).await?;
            rally_view_state(&pool, user_id, rally_result_id, require_user_result).await
        }
    }
}
