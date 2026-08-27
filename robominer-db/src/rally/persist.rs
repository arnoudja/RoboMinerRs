use sqlx::MySqlPool;

use crate::mappers::{MiningRallyQueueRow, mining_rally_queue_rows};
use crate::{
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord, DbOutcome, MiningRallyQueueRecord, PersistRallyRejection, db_ok,
    db_reject,
};

/// How long a worker holds unfinished queue rows while simulating.
pub const PROCESSING_LEASE_SECONDS: i32 = 1_800;

pub async fn persist_completed_rally(
    pool: &MySqlPool,
    rally: &CompletedRallyRecord,
) -> Result<DbOutcome<i64, PersistRallyRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let result = sqlx::query("INSERT INTO RallyResult (resultData) VALUES (?)")
        .bind(&rally.result_data)
        .execute(&mut *transaction)
        .await?;
    let rally_result_id = result.last_insert_id() as i64;

    for participant in &rally.participants {
        update_robot_for_completed_rally(&mut transaction, participant).await?;
        if !update_mining_queue_for_completed_rally(&mut transaction, participant, rally_result_id)
            .await?
        {
            transaction.rollback().await?;
            return db_reject(PersistRallyRejection::QueueAlreadyFinished);
        }
        apply_pending_robot_changes(&mut transaction, participant).await?;
        super::cleanup::cleanup_old_claimed_mining_queue_items(
            &mut transaction,
            participant.robot_id,
        )
        .await?;

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

    db_ok(rally_result_id)
}

/// Lock and lease the next ready rally queue for an area.
///
/// Uses `FOR UPDATE SKIP LOCKED` so concurrent workers skip rows already locked.
/// Avoids MySQL-only `FOR UPDATE OF table` (unsupported on MariaDB). Sets
/// `processingLeaseUntil` before commit so unlocked readers also skip in-flight
/// work. Does not hold row locks across simulation.
pub async fn claim_next_mining_rally_queue_for_area(
    pool: &MySqlPool,
    mining_area_id: i64,
    rally_size: usize,
    expiry_start_seconds: i32,
) -> Result<Option<Vec<MiningRallyQueueRecord>>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query_as::<_, MiningRallyQueueRow>(
        "SELECT MiningQueue.id, MiningQueue.miningAreaId, MiningQueue.robotId, \
                Robot.userId, \
                MiningQueue.rallyResultId, MiningQueue.playerNumber, MiningQueue.score, \
                MiningQueue.claimed, \
                TIMESTAMPDIFF(SECOND, NOW(), \
                    TIMESTAMPADD(SECOND, MiningArea.miningTime, \
                        IF(Robot.rechargeEndTime < MiningQueue.creationTime, \
                           MiningQueue.creationTime, Robot.rechargeEndTime))) AS secondsLeft \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE MiningQueue.miningAreaId = ? \
           AND MiningQueue.miningEndTime IS NULL \
           AND (MiningQueue.processingLeaseUntil IS NULL \
                OR MiningQueue.processingLeaseUntil < NOW()) \
           AND (Robot.rechargeEndTime IS NULL OR Robot.rechargeEndTime <= NOW()) \
           AND (Robot.miningEndTime IS NULL OR Robot.miningEndTime <= NOW()) \
           AND NOT EXISTS ( \
               SELECT prev.id \
               FROM MiningQueue prev \
               WHERE prev.id < MiningQueue.id \
                 AND prev.robotId = MiningQueue.robotId \
                 AND prev.miningEndTime IS NULL \
           ) \
         ORDER BY secondsLeft, MiningQueue.id \
         FOR UPDATE SKIP LOCKED",
    )
    .bind(mining_area_id)
    .fetch_all(&mut *transaction)
    .await?;

    let mut queue_rows = mining_rally_queue_rows(rows);
    let ready = !queue_rows.is_empty()
        && (queue_rows.len() >= rally_size || queue_rows[0].seconds_left < expiry_start_seconds);
    if !ready {
        transaction.rollback().await?;
        return Ok(None);
    }

    if queue_rows.len() > rally_size {
        queue_rows.truncate(rally_size);
    }

    for row in &queue_rows {
        sqlx::query(
            "UPDATE MiningQueue \
             SET processingLeaseUntil = TIMESTAMPADD(SECOND, ?, NOW()) \
             WHERE id = ?",
        )
        .bind(PROCESSING_LEASE_SECONDS)
        .bind(row.queue.id)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(Some(queue_rows))
}

/// Queue heads across all areas for predicting the next claimable rally delay.
///
/// Includes robots that are still mining/recharging and rows with an active
/// processing lease; callers use `busy_seconds` (lease + robot free times) to
/// decide when each head becomes claimable.
pub async fn list_next_claim_rally_candidates(
    pool: &MySqlPool,
) -> Result<Vec<crate::NextClaimRallyCandidate>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i32, i32)>(
        "SELECT MiningQueue.miningAreaId, \
                GREATEST( \
                    0, \
                    COALESCE(TIMESTAMPDIFF(SECOND, NOW(), Robot.miningEndTime), 0), \
                    COALESCE(TIMESTAMPDIFF(SECOND, NOW(), Robot.rechargeEndTime), 0), \
                    COALESCE(TIMESTAMPDIFF(SECOND, NOW(), MiningQueue.processingLeaseUntil), 0) \
                ) AS busySeconds, \
                TIMESTAMPDIFF(SECOND, NOW(), \
                    TIMESTAMPADD(SECOND, MiningArea.miningTime, \
                        IF(Robot.rechargeEndTime < MiningQueue.creationTime, \
                           MiningQueue.creationTime, Robot.rechargeEndTime))) AS secondsLeft \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         INNER JOIN MiningArea ON MiningArea.id = MiningQueue.miningAreaId \
         WHERE MiningQueue.miningEndTime IS NULL \
           AND NOT EXISTS ( \
               SELECT prev.id \
               FROM MiningQueue prev \
               WHERE prev.id < MiningQueue.id \
                 AND prev.robotId = MiningQueue.robotId \
                 AND prev.miningEndTime IS NULL \
           ) \
         ORDER BY MiningQueue.miningAreaId, busySeconds, secondsLeft, MiningQueue.id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, busy_seconds, seconds_left)| crate::NextClaimRallyCandidate {
                    mining_area_id,
                    busy_seconds,
                    seconds_left,
                },
            )
            .collect()
    })
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

/// Returns `true` when the queue row was still unfinished and was updated.
async fn update_mining_queue_for_completed_rally(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    participant: &CompletedRallyParticipantRecord,
    rally_result_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE MiningQueue \
         SET rallyResultId = ?, \
             miningEndTime = TIMESTAMPADD(SECOND, ?, NOW()), \
             playerNumber = ?, \
             executedSourceCode = ?, \
             processingLeaseUntil = NULL \
         WHERE id = ? \
           AND miningEndTime IS NULL",
    )
    .bind(rally_result_id)
    .bind(participant.mining_end_seconds_from_now)
    .bind(participant.player_number)
    .bind(&participant.executed_source_code)
    .bind(participant.mining_queue_id)
    .execute(&mut **transaction)
    .await?;

    Ok(result.rows_affected() == 1)
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
        "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, depotAmount) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(mining_queue_id)
    .bind(ore_result.ore_id)
    .bind(ore_result.amount)
    .bind(ore_result.depot_amount.clamp(0, ore_result.amount))
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
    let updated_score = super::score::updated_robot_mining_area_score(
        previous.map(|(_, score)| score),
        participant.score,
    );

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
