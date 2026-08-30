use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::{ensure_destructive_confirmed, ensure_positive_user_id};
use crate::achievement::{achievement_page_states, achievement_states, claim_achievement_step};
use crate::cli::AchievementCommand;
use crate::database::connect_database;

pub(crate) async fn dispatch_achievement(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: AchievementCommand,
) -> Result<()> {
    match command {
        AchievementCommand::ClaimStep {
            user_id,
            achievement_id,
            i_understand,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure_destructive_confirmed(i_understand, "achievement claim-step")?;
            ensure!(
                achievement_id > 0,
                "--achievement-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            claim_achievement_step(
                &pool,
                robominer_db::ClaimAchievementStepRequest {
                    user_id,
                    achievement_id,
                },
            )
            .await
        }
        AchievementCommand::States { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            achievement_states(&pool, user_id).await
        }
        AchievementCommand::PageStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            achievement_page_states(&pool, user_id).await
        }
    }
}
