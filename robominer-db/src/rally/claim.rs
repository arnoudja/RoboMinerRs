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

    for queue in &claimable_queues {
        let queue_rewards = claim_mining_queue(&mut transaction, queue).await?;
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

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningQueue {
    mining_queue_id: i64,
    mining_area_id: i64,
    robot_id: i64,
    robot_max_ore: i32,
}

#[derive(Debug, Clone, Copy)]
struct ClaimableMiningOreResult {
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
    let mut rewards: Vec<ClaimedOreRewardRecord> = Vec::new();

    for (ore_id, reward) in ore_rewards {
        if reward <= 0 {
            continue;
        }
        let ore_name: String = sqlx::query_scalar("SELECT oreName FROM Ore WHERE id = ?")
            .bind(ore_id)
            .fetch_one(&mut **transaction)
            .await?;
        rewards.push(ClaimedOreRewardRecord {
            ore_id,
            ore_name,
            reward,
        });
    }

    rewards.sort_by_key(|reward| std::cmp::Reverse(reward.ore_id));
    Ok(rewards)
}

async fn claim_mining_queue(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    queue: &ClaimableMiningQueue,
) -> Result<Vec<(i64, i32)>, sqlx::Error> {
    sqlx::query("UPDATE Robot SET totalMiningRuns = totalMiningRuns + 1 WHERE id = ?")
        .bind(queue.robot_id)
        .execute(&mut **transaction)
        .await?;

    sqlx::query("UPDATE MiningQueue SET claimed = true WHERE id = ?")
        .bind(queue.mining_queue_id)
        .execute(&mut **transaction)
        .await?;

    calculate_mining_ore_result_tax(transaction, queue.mining_queue_id).await?;
    let ore_results = list_claimable_mining_ore_results(transaction, queue.mining_queue_id).await?;

    for ore_result in &ore_results {
        upsert_robot_lifetime_result(transaction, queue.robot_id, ore_result).await?;
        upsert_user_ore_asset_from_reward(transaction, queue.robot_id, ore_result).await?;
    }

    update_mining_area_lifetime_results(transaction, queue, &ore_results).await?;

    Ok(ore_results
        .iter()
        .map(|ore_result| (ore_result.ore_id, ore_result.amount - ore_result.tax))
        .filter(|(_, reward)| *reward > 0)
        .collect())
}

async fn calculate_mining_ore_result_tax(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    mining_queue_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE MiningOreResult \
         INNER JOIN MiningQueue ON MiningQueue.id = MiningOreResult.miningQueueId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         SET MiningOreResult.tax = FLOOR(MiningOreResult.amount * MiningArea.taxRate / 100) \
         WHERE MiningOreResult.miningQueueId = ?",
    )
    .bind(mining_queue_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn list_claimable_mining_ore_results(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    mining_queue_id: i64,
) -> Result<Vec<ClaimableMiningOreResult>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32, i32)>(
        "SELECT oreId, amount, COALESCE(tax, 0) \
         FROM MiningOreResult \
         WHERE miningQueueId = ? \
         ORDER BY oreId",
    )
    .bind(mining_queue_id)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(ore_id, amount, tax)| ClaimableMiningOreResult {
            ore_id,
            amount,
            tax,
        })
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

    for ore_id in ore_ids {
        let amount = ore_results
            .iter()
            .find(|ore_result| ore_result.ore_id == ore_id)
            .map(|ore_result| ore_result.amount)
            .unwrap_or(0);

        sqlx::query(
            "INSERT INTO MiningAreaLifetimeResult \
             (miningAreaId, oreId, totalAmount, totalContainerSize) \
             VALUES (?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE \
             totalAmount = totalAmount + VALUES(totalAmount), \
             totalContainerSize = totalContainerSize + VALUES(totalContainerSize)",
        )
        .bind(queue.mining_area_id)
        .bind(ore_id)
        .bind(amount)
        .bind(queue.robot_max_ore)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}
