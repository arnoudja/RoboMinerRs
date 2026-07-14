use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::{
    ClaimedMiningQueueCleanupSummary, ClaimedOreRewardRecord, ClaimedUserResults,
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord, INITIAL_ORE_WALLET_MAX, SCORE_HISTORY_FACTOR, SCORE_START_FACTOR,
};

const CLAIMED_MINING_QUEUE_RETENTION: i64 = 12;

pub async fn persist_completed_rally(
    pool: &MySqlPool,
    rally: &CompletedRallyRecord,
) -> Result<i64, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let result = sqlx::query("INSERT INTO RallyResult (resultData) VALUES (?)")
        .bind(&rally.result_data)
        .execute(&mut *transaction)
        .await?;
    let rally_result_id = result.last_insert_id() as i64;

    for participant in &rally.participants {
        update_robot_for_completed_rally(&mut transaction, participant).await?;
        update_mining_queue_for_completed_rally(&mut transaction, participant, rally_result_id)
            .await?;
        apply_pending_robot_changes(&mut transaction, participant).await?;
        cleanup_old_claimed_mining_queue_items(&mut transaction, participant.robot_id).await?;

        for ore_result in participant
            .ore_results
            .iter()
            .filter(|ore_result| ore_result.amount > 0)
        {
            insert_mining_ore_result(&mut transaction, participant.mining_queue_id, ore_result)
                .await?;
        }

        for action_result in participant
            .action_results
            .iter()
            .filter(|action_result| action_result.amount > 0)
        {
            insert_robot_action_result(
                &mut transaction,
                participant.mining_queue_id,
                action_result,
            )
            .await?;
        }

        update_robot_mining_area_score(&mut transaction, participant).await?;
    }

    transaction.commit().await?;

    Ok(rally_result_id)
}
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

    reconcile_pending_robot_changes_in_transaction(&mut transaction, user_id).await?;
    let ore_rewards = load_claimed_ore_rewards(&mut transaction, ore_rewards).await?;
    transaction.commit().await?;

    Ok(ClaimedUserResults {
        claimed_queues,
        ore_rewards,
    })
}
pub fn updated_robot_mining_area_score(previous_score: Option<f64>, score: f64) -> f64 {
    match previous_score {
        Some(previous_score) => {
            ((SCORE_HISTORY_FACTOR - 1.0) * previous_score + score) / SCORE_HISTORY_FACTOR
        }
        None => score / SCORE_START_FACTOR,
    }
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

pub async fn reconcile_pending_robot_changes_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    reconcile_pending_robot_changes_in_transaction(&mut transaction, user_id).await?;
    transaction.commit().await?;
    Ok(())
}

async fn reconcile_pending_robot_changes_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    apply_orphaned_pending_robot_changes(transaction, user_id).await?;
    delete_committed_pending_robot_changes(transaction, user_id).await?;
    Ok(())
}

async fn apply_orphaned_pending_robot_changes(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Robot \
         INNER JOIN PendingRobotChanges \
         ON PendingRobotChanges.robotId = Robot.id \
         SET Robot.sourceCode = PendingRobotChanges.sourceCode, \
             Robot.oreContainerId = PendingRobotChanges.oreContainerId, \
             Robot.miningUnitId = PendingRobotChanges.miningUnitId, \
             Robot.batteryId = PendingRobotChanges.batteryId, \
             Robot.memoryModuleId = PendingRobotChanges.memoryModuleId, \
             Robot.cpuId = PendingRobotChanges.cpuId, \
             Robot.engineId = PendingRobotChanges.engineId, \
             Robot.oreScannerId = PendingRobotChanges.oreScannerId, \
             Robot.rechargeTime = PendingRobotChanges.rechargeTime, \
             Robot.maxOre = PendingRobotChanges.maxOre, \
             Robot.miningSpeed = PendingRobotChanges.miningSpeed, \
             Robot.maxTurns = PendingRobotChanges.maxTurns, \
             Robot.memorySize = PendingRobotChanges.memorySize, \
             Robot.cpuSpeed = PendingRobotChanges.cpuSpeed, \
             Robot.forwardSpeed = PendingRobotChanges.forwardSpeed, \
             Robot.backwardSpeed = PendingRobotChanges.backwardSpeed, \
             Robot.rotateSpeed = PendingRobotChanges.rotateSpeed, \
             Robot.robotSize = PendingRobotChanges.robotSize, \
             Robot.scanTime = PendingRobotChanges.scanTime, \
             Robot.scanDistance = PendingRobotChanges.scanDistance, \
             PendingRobotChanges.changesCommitTime = NOW() \
         WHERE Robot.userId = ? \
           AND PendingRobotChanges.changesCommitTime IS NULL \
           AND NOT EXISTS ( \
               SELECT 1 \
               FROM MiningQueue \
               WHERE MiningQueue.robotId = Robot.id \
                 AND (MiningQueue.miningEndTime IS NULL \
                      OR MiningQueue.miningEndTime > NOW()) \
           )",
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn delete_committed_pending_robot_changes(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE PendingRobotChanges \
         FROM PendingRobotChanges \
         INNER JOIN Robot ON Robot.id = PendingRobotChanges.robotId \
         WHERE Robot.userId = ? \
           AND PendingRobotChanges.changesCommitTime <= NOW()",
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn cleanup_old_claimed_mining_queue_items_for_robot(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<ClaimedMiningQueueCleanupSummary, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let summary = cleanup_old_claimed_mining_queue_items(&mut transaction, robot_id).await?;
    transaction.commit().await?;

    Ok(summary)
}

async fn cleanup_old_claimed_mining_queue_items(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    robot_id: i64,
) -> Result<ClaimedMiningQueueCleanupSummary, sqlx::Error> {
    let old_items = sqlx::query_as::<_, (i64, Option<i64>)>(
        "SELECT id, rallyResultId \
         FROM MiningQueue \
         WHERE robotId = ? \
           AND claimed = true \
         ORDER BY id DESC \
         LIMIT ?, 100000",
    )
    .bind(robot_id)
    .bind(CLAIMED_MINING_QUEUE_RETENTION)
    .fetch_all(&mut **transaction)
    .await?;

    let mut summary = ClaimedMiningQueueCleanupSummary::default();

    for (mining_queue_id, rally_result_id) in old_items {
        sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(mining_queue_id)
            .execute(&mut **transaction)
            .await?;
        summary.queues_deleted += 1;

        if let Some(rally_result_id) = rally_result_id
            && !rally_result_still_referenced(transaction, rally_result_id).await?
        {
            sqlx::query("DELETE FROM RallyResult WHERE id = ?")
                .bind(rally_result_id)
                .execute(&mut **transaction)
                .await?;
            summary.rally_results_deleted += 1;
        }
    }

    Ok(summary)
}

async fn rally_result_still_referenced(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    rally_result_id: i64,
) -> Result<bool, sqlx::Error> {
    let referenced: Option<i64> =
        sqlx::query_scalar("SELECT id FROM MiningQueue WHERE rallyResultId = ? LIMIT 1")
            .bind(rally_result_id)
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(referenced.is_some())
}

async fn update_robot_for_completed_rally(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    participant: &CompletedRallyParticipantRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Robot \
         SET miningEndTime = TIMESTAMPADD(SECOND, ?, NOW()), \
             rechargeEndTime = TIMESTAMPADD(SECOND, rechargeTime, TIMESTAMPADD(SECOND, ?, NOW())) \
         WHERE id = ?",
    )
    .bind(participant.mining_end_seconds_from_now)
    .bind(participant.mining_end_seconds_from_now)
    .bind(participant.robot_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn update_mining_queue_for_completed_rally(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    participant: &CompletedRallyParticipantRecord,
    rally_result_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE MiningQueue \
         SET rallyResultId = ?, \
             miningEndTime = TIMESTAMPADD(SECOND, ?, NOW()), \
             playerNumber = ? \
         WHERE id = ?",
    )
    .bind(rally_result_id)
    .bind(participant.mining_end_seconds_from_now)
    .bind(participant.player_number)
    .bind(participant.mining_queue_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn apply_pending_robot_changes(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    participant: &CompletedRallyParticipantRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE Robot \
         INNER JOIN PendingRobotChanges \
         ON PendingRobotChanges.robotId = Robot.id \
         SET Robot.sourceCode = PendingRobotChanges.sourceCode, \
             Robot.oreContainerId = PendingRobotChanges.oreContainerId, \
             Robot.miningUnitId = PendingRobotChanges.miningUnitId, \
             Robot.batteryId = PendingRobotChanges.batteryId, \
             Robot.memoryModuleId = PendingRobotChanges.memoryModuleId, \
             Robot.cpuId = PendingRobotChanges.cpuId, \
             Robot.engineId = PendingRobotChanges.engineId, \
             Robot.oreScannerId = PendingRobotChanges.oreScannerId, \
             Robot.rechargeTime = PendingRobotChanges.rechargeTime, \
             Robot.maxOre = PendingRobotChanges.maxOre, \
             Robot.miningSpeed = PendingRobotChanges.miningSpeed, \
             Robot.maxTurns = PendingRobotChanges.maxTurns, \
             Robot.memorySize = PendingRobotChanges.memorySize, \
             Robot.cpuSpeed = PendingRobotChanges.cpuSpeed, \
             Robot.forwardSpeed = PendingRobotChanges.forwardSpeed, \
             Robot.backwardSpeed = PendingRobotChanges.backwardSpeed, \
             Robot.rotateSpeed = PendingRobotChanges.rotateSpeed, \
             Robot.robotSize = PendingRobotChanges.robotSize, \
             Robot.scanTime = PendingRobotChanges.scanTime, \
             Robot.scanDistance = PendingRobotChanges.scanDistance, \
             PendingRobotChanges.changesCommitTime = TIMESTAMPADD(SECOND, ?, NOW()) \
         WHERE Robot.id = ?",
    )
    .bind(participant.mining_end_seconds_from_now)
    .bind(participant.robot_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn insert_mining_ore_result(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    mining_queue_id: i64,
    ore_result: &CompletedRallyOreRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO MiningOreResult (miningQueueId, oreId, amount) \
         VALUES (?, ?, ?)",
    )
    .bind(mining_queue_id)
    .bind(ore_result.ore_id)
    .bind(ore_result.amount)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn insert_robot_action_result(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    mining_queue_id: i64,
    action_result: &CompletedRallyActionRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO RobotActionsDone (miningQueueId, actionType, amount) \
         VALUES (?, ?, ?)",
    )
    .bind(mining_queue_id)
    .bind(action_result.action_type)
    .bind(action_result.amount)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn update_robot_mining_area_score(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    participant: &CompletedRallyParticipantRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE MiningQueue SET score = ? WHERE id = ?")
        .bind(participant.score)
        .bind(participant.mining_queue_id)
        .execute(&mut **transaction)
        .await?;

    let previous = sqlx::query_as::<_, (i32, f64)>(
        "SELECT totalRuns, score \
         FROM RobotMiningAreaScore \
         WHERE robotId = ? AND miningAreaId = ? \
         FOR UPDATE",
    )
    .bind(participant.robot_id)
    .bind(participant.mining_area_id)
    .fetch_optional(&mut **transaction)
    .await?;
    let updated_score =
        updated_robot_mining_area_score(previous.map(|(_, score)| score), participant.score);

    if previous.is_some() {
        sqlx::query(
            "UPDATE RobotMiningAreaScore \
             SET score = ?, totalRuns = totalRuns + 1 \
             WHERE robotId = ? AND miningAreaId = ?",
        )
        .bind(updated_score)
        .bind(participant.robot_id)
        .bind(participant.mining_area_id)
        .execute(&mut **transaction)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
             VALUES (?, ?, 1, ?)",
        )
        .bind(participant.robot_id)
        .bind(participant.mining_area_id)
        .bind(updated_score)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::updated_robot_mining_area_score;

    #[test]
    fn new_robot_mining_area_scores_start_below_raw_score() {
        assert_eq!(updated_robot_mining_area_score(None, 140.0), 100.0);
    }

    #[test]
    fn existing_robot_mining_area_scores_use_legacy_history_factor() {
        assert_eq!(updated_robot_mining_area_score(Some(100.0), 150.0), 110.0);
    }
}
