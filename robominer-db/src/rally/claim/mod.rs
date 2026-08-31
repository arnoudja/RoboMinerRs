mod batch;
mod deadlock;
mod queries;
mod types;

use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::ClaimedUserResults;

pub use queries::{
    count_claimable_mining_queues, list_user_ids_with_claimable_mining_queues,
    next_wallet_claim_delay_seconds,
};

use batch::{claim_mining_queues_batch, load_claimed_ore_rewards};
use deadlock::{MAX_CLAIM_DEADLOCK_ATTEMPTS, is_mysql_deadlock};
use queries::list_claimable_mining_queues;

pub async fn claim_user_results(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<ClaimedUserResults, sqlx::Error> {
    let mut attempt = 0;
    loop {
        match claim_user_results_once(pool, user_id).await {
            Ok(result) => return Ok(result),
            Err(error)
                if is_mysql_deadlock(&error) && attempt + 1 < MAX_CLAIM_DEADLOCK_ATTEMPTS =>
            {
                attempt += 1;
                tracing::debug!(
                    attempt,
                    user_id,
                    "retrying mining wallet claim after deadlock"
                );
                tokio::task::yield_now().await;
            }
            Err(error) => return Err(error),
        }
    }
}

async fn claim_user_results_once(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<ClaimedUserResults, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let claimable_queues = list_claimable_mining_queues(&mut transaction, user_id).await?;
    let claimed_queues = claimable_queues.len() as u64;
    let mut ore_rewards: HashMap<i64, i32> = HashMap::new();

    if !claimable_queues.is_empty() {
        let queue_rewards = claim_mining_queues_batch(&mut transaction, &claimable_queues).await?;
        for (ore_id, reward) in queue_rewards {
            *ore_rewards.entry(ore_id).or_default() += reward;
        }
    }

    super::pending::reconcile_pending_robot_changes_in_transaction(&mut transaction, user_id)
        .await?;
    let ore_rewards = load_claimed_ore_rewards(&mut transaction, ore_rewards).await?;
    transaction.commit().await?;

    Ok(ClaimedUserResults {
        claimed_queues,
        ore_rewards,
    })
}
