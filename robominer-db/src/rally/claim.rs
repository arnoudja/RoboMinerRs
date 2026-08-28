use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::{ClaimedOreRewardRecord, ClaimedUserResults, INITIAL_ORE_WALLET_MAX};

pub async fn claim_user_results(
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

/// Read-only count of finished mining runs waiting to be claimed into the wallet.
pub async fn count_claimable_mining_queues(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND Robot.userId = ? \
           AND MiningQueue.claimed = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count.max(0) as u64)
}

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningQueue {
    mining_queue_id: i64,
    mining_area_id: i64,
    robot_id: i64,
    robot_max_ore: i32,
}

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningOreResult {
    mining_queue_id: i64,
    ore_id: i64,
    amount: i32,
    tax: i32,
}

async fn list_claimable_mining_queues(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<Vec<ClaimableMiningQueue>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, i32)>(
        "SELECT MiningQueue.id, MiningQueue.miningAreaId, MiningQueue.robotId, Robot.maxOre \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND Robot.userId = ? \
           AND MiningQueue.claimed = false \
         ORDER BY MiningQueue.miningEndTime, MiningQueue.id \
         FOR UPDATE",
    )
    .bind(user_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(mining_queue_id, mining_area_id, robot_id, robot_max_ore)| ClaimableMiningQueue {
                mining_queue_id,
                mining_area_id,
                robot_id,
                robot_max_ore,
            },
        )
        .collect())
}

async fn load_claimed_ore_rewards(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ore_rewards: HashMap<i64, i32>,
) -> Result<Vec<ClaimedOreRewardRecord>, sqlx::Error> {
    let positive: Vec<(i64, i32)> = ore_rewards
        .into_iter()
        .filter(|(_, reward)| *reward > 0)
        .collect();
    if positive.is_empty() {
        return Ok(Vec::new());
    }

    let ore_ids: Vec<i64> = positive.iter().map(|(ore_id, _)| *ore_id).collect();
    let placeholders = vec!["?"; ore_ids.len()].join(", ");
    let query = format!("SELECT id, oreName FROM Ore WHERE id IN ({placeholders})");
    let mut query_builder = sqlx::query_as::<_, (i64, String)>(&query);
    for ore_id in &ore_ids {
        query_builder = query_builder.bind(ore_id);
    }
    let names: HashMap<i64, String> = query_builder
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .collect();

    let mut rewards: Vec<ClaimedOreRewardRecord> = Vec::with_capacity(positive.len());
    for (ore_id, reward) in positive {
        let Some(ore_name) = names.get(&ore_id).cloned() else {
            return Err(sqlx::Error::RowNotFound);
        };
        rewards.push(ClaimedOreRewardRecord {
            ore_id,
            ore_name,
            reward,
        });
    }

    rewards.sort_by_key(|reward| std::cmp::Reverse(reward.ore_id));
    Ok(rewards)
}

async fn claim_mining_queues_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    claimable_queues: &[ClaimableMiningQueue],
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    let queue_ids: Vec<i64> = claimable_queues
        .iter()
        .map(|queue| queue.mining_queue_id)
        .collect();

    increment_robot_mining_runs_batch(transaction, claimable_queues).await?;
    mark_mining_queues_claimed_batch(transaction, &queue_ids).await?;
    calculate_mining_ore_result_tax_batch(transaction, &queue_ids).await?;
    let ore_results = list_claimable_mining_ore_results_batch(transaction, &queue_ids).await?;

    let mut ore_results_by_queue: HashMap<i64, Vec<ClaimableMiningOreResult>> = HashMap::new();
    for ore_result in ore_results {
        ore_results_by_queue
            .entry(ore_result.mining_queue_id)
            .or_default()
            .push(ore_result);
    }

    let mut rewards = Vec::new();
    for queue in claimable_queues {
        let queue_ore_results = ore_results_by_queue
            .remove(&queue.mining_queue_id)
            .unwrap_or_default();

        for ore_result in &queue_ore_results {
            upsert_robot_lifetime_result(transaction, queue.robot_id, ore_result).await?;
            upsert_user_ore_asset_from_reward(transaction, queue.robot_id, ore_result).await?;
        }

        update_mining_area_lifetime_results(transaction, queue, &queue_ore_results).await?;

        for ore_result in &queue_ore_results {
            let reward = ore_result.amount - ore_result.tax;
            if reward > 0 {
                rewards.push((ore_result.ore_id, reward));
            }
        }
    }

    Ok(rewards)
}

async fn increment_robot_mining_runs_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    claimable_queues: &[ClaimableMiningQueue],
) -> Result<(), sqlx::Error> {
    let mut run_counts: HashMap<i64, u32> = HashMap::new();
    for queue in claimable_queues {
        *run_counts.entry(queue.robot_id).or_default() += 1;
    }

    for (robot_id, count) in run_counts {
        sqlx::query("UPDATE Robot SET totalMiningRuns = totalMiningRuns + ? WHERE id = ?")
            .bind(count)
            .bind(robot_id)
            .execute(&mut **transaction)
            .await?;
    }

    Ok(())
}

async fn mark_mining_queues_claimed_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(());
    }

    let placeholders = vec!["?"; queue_ids.len()].join(", ");
    let query = format!("UPDATE MiningQueue SET claimed = true WHERE id IN ({placeholders})");
    let mut query_builder = sqlx::query(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    query_builder.execute(&mut **transaction).await?;

    Ok(())
}

async fn calculate_mining_ore_result_tax_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<(), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(());
    }

    let placeholders = vec!["?"; queue_ids.len()].join(", ");
    let query = format!(
        "UPDATE MiningOreResult \
         INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         SET MiningOreResult.tax = \
             FLOOR(GREATEST(MiningOreResult.amount - MiningOreResult.depotAmount, 0) \
                   * MiningArea.taxRate / 100) \
           + FLOOR(LEAST(MiningOreResult.depotAmount, MiningOreResult.amount) \
                   * MiningArea.depotTaxRate / 100) \
         WHERE MiningOreResult.miningQueueId IN ({placeholders})"
    );
    let mut query_builder = sqlx::query(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    query_builder.execute(&mut **transaction).await?;

    Ok(())
}

async fn list_claimable_mining_ore_results_batch(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue_ids: &[i64],
) -> Result<Vec<ClaimableMiningOreResult>, sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = vec!["?"; queue_ids.len()].join(", ");
    let query = format!(
        "SELECT miningQueueId, oreId, amount, COALESCE(tax, 0) \
         FROM MiningOreResult \
         WHERE miningQueueId IN ({placeholders}) \
         ORDER BY miningQueueId, oreId"
    );
    let mut query_builder = sqlx::query_as::<_, (i64, i64, i32, i32)>(&query);
    for queue_id in queue_ids {
        query_builder = query_builder.bind(queue_id);
    }
    let rows = query_builder.fetch_all(&mut **transaction).await?;

    Ok(rows
        .into_iter()
        .map(
            |(mining_queue_id, ore_id, amount, tax)| ClaimableMiningOreResult {
                mining_queue_id,
                ore_id,
                amount,
                tax,
            },
        )
        .collect())
}

async fn upsert_robot_lifetime_result(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    ore_result: &ClaimableMiningOreResult,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO RobotLifetimeResult (robotId, oreId, amount, tax) \
         VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         amount = amount + VALUES(amount), \
         tax = tax + VALUES(tax)",
    )
    .bind(robot_id)
    .bind(ore_result.ore_id)
    .bind(ore_result.amount)
    .bind(ore_result.tax)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn upsert_user_ore_asset_from_reward(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
    ore_result: &ClaimableMiningOreResult,
) -> Result<(), sqlx::Error> {
    let reward = ore_result.amount - ore_result.tax;

    sqlx::query(
        "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
         SELECT Robot.userId, ?, LEAST(?, ?), ? \
         FROM Robot \
         WHERE Robot.id = ? \
         ON DUPLICATE KEY UPDATE \
         amount = LEAST(maxAllowed, amount + ?)",
    )
    .bind(ore_result.ore_id)
    .bind(reward)
    .bind(INITIAL_ORE_WALLET_MAX)
    .bind(INITIAL_ORE_WALLET_MAX)
    .bind(robot_id)
    .bind(reward)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn update_mining_area_lifetime_results(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue: &ClaimableMiningQueue,
    ore_results: &[ClaimableMiningOreResult],
) -> Result<(), sqlx::Error> {
    let ore_ids = sqlx::query_scalar::<_, i64>(
        "SELECT DISTINCT oreId \
         FROM MiningAreaOreSupply \
         WHERE miningAreaId = ? \
         ORDER BY oreId",
    )
    .bind(queue.mining_area_id)
    .fetch_all(&mut **transaction)
    .await?;

    sqlx::query(
        "UPDATE MiningAreaLifetimeResult \
         SET totalRuns = totalRuns + 1 \
         WHERE miningAreaId = ?",
    )
    .bind(queue.mining_area_id)
    .execute(&mut **transaction)
    .await?;

    for ore_id in ore_ids {
        let amount = ore_results
            .iter()
            .find(|ore_result| ore_result.ore_id == ore_id)
            .map(|ore_result| ore_result.amount)
            .unwrap_or(0);

        sqlx::query(
            "INSERT INTO MiningAreaLifetimeResult \
             (miningAreaId, oreId, totalAmount, totalContainerSize, totalRuns) \
             VALUES (?, ?, ?, ?, \
                     COALESCE((SELECT totalRuns \
                               FROM MiningAreaLifetimeResult AS existing \
                               WHERE existing.miningAreaId = ? \
                               LIMIT 1), 1)) \
             ON DUPLICATE KEY UPDATE \
             totalAmount = totalAmount + VALUES(totalAmount), \
             totalContainerSize = totalContainerSize + VALUES(totalContainerSize)",
        )
        .bind(queue.mining_area_id)
        .bind(ore_id)
        .bind(amount)
        .bind(queue.robot_max_ore)
        .bind(queue.mining_area_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}
