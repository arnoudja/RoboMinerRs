use sqlx::MySqlPool;

pub async fn reconcile_pending_robot_changes_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    reconcile_pending_robot_changes_in_transaction(&mut transaction, user_id).await?;
    transaction.commit().await?;
    Ok(())
}

pub(super) async fn reconcile_pending_robot_changes_in_transaction(
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
