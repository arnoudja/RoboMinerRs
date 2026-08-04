use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::ensure_positive_user_id;
use crate::cli::MiningCommand;
use crate::database::connect_database;
use crate::mining::{
    cancel_mining_queue, claim_results, enqueue_mining, mining_area_overview_states,
    mining_area_scores, mining_queue_page_states, mining_queue_states, mining_result_states,
};

pub(crate) async fn dispatch_mining(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: MiningCommand,
) -> Result<()> {
    match command {
        MiningCommand::ClaimResults { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            claim_results(&pool, user_id).await
        }
        MiningCommand::Enqueue {
            user_id,
            robot_id,
            mining_area_id,
            fill,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(robot_id > 0, "--robot-id must be greater than zero");
            ensure!(
                mining_area_id > 0,
                "--mining-area-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            enqueue_mining(
                &pool,
                robominer_db::EnqueueMiningRequest {
                    user_id,
                    robot_id,
                    mining_area_id,
                    fill,
                },
            )
            .await
        }
        MiningCommand::CancelQueue {
            user_id,
            mining_queue_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                mining_queue_id > 0,
                "--mining-queue-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            cancel_mining_queue(
                &pool,
                robominer_db::CancelMiningQueueRequest {
                    user_id,
                    mining_queue_id,
                    require_refund_fits: false,
                },
            )
            .await
        }
        MiningCommand::QueueStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_queue_states(&pool, user_id).await
        }
        MiningCommand::QueuePageStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_queue_page_states(&pool, user_id).await
        }
        MiningCommand::AreaScores { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            mining_area_scores(&pool, user_id).await
        }
        MiningCommand::ResultStates {
            user_id,
            max_results,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(max_results > 0, "--max-results must be greater than zero");
            let pool = connect_database(database_url, config).await?;
            mining_result_states(&pool, user_id, max_results).await
        }
        MiningCommand::AreaOverviewStates => {
            let pool = connect_database(database_url, config).await?;
            mining_area_overview_states(&pool).await
        }
    }
}
