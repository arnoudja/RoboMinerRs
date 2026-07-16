use sqlx::MySqlPool;

use crate::{
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord,
};

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

    Ok(rally_result_id)
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
