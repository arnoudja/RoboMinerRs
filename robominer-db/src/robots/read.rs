use sqlx::MySqlPool;

use crate::mappers::{robot_config_state_record, robot_record};
use crate::{
    RobotConfigPartAssetStateRecord, RobotConfigStateRecord, RobotLifetimeOreStatRecord,
    RobotMiningAreaScoreRecord, RobotMiningAreaStatRecord, RobotRecord, RobotStatsHeaderRecord,
};
pub async fn list_robot_config_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotConfigStateRecord>, sqlx::Error> {
    crate::reconcile_pending_robot_changes_for_user(pool, user_id).await?;

    let rows = sqlx::query(
        "SELECT Robot.id AS robotId, \
                Robot.robotName, \
                Robot.programSourceId, \
                COALESCE(PendingRobotChanges.oreContainerId, Robot.oreContainerId) AS oreContainerId, \
                OreContainer.partName AS oreContainerName, \
                COALESCE(PendingRobotChanges.miningUnitId, Robot.miningUnitId) AS miningUnitId, \
                MiningUnit.partName AS miningUnitName, \
                COALESCE(PendingRobotChanges.batteryId, Robot.batteryId) AS batteryId, \
                Battery.partName AS batteryName, \
                COALESCE(PendingRobotChanges.memoryModuleId, Robot.memoryModuleId) AS memoryModuleId, \
                MemoryModule.partName AS memoryModuleName, \
                COALESCE(PendingRobotChanges.cpuId, Robot.cpuId) AS cpuId, \
                Cpu.partName AS cpuName, \
                COALESCE(PendingRobotChanges.engineId, Robot.engineId) AS engineId, \
                Engine.partName AS engineName, \
                COALESCE(PendingRobotChanges.oreScannerId, Robot.oreScannerId) AS oreScannerId, \
                OreScanner.partName AS oreScannerName, \
                COALESCE(PendingRobotChanges.rechargeTime, Robot.rechargeTime) AS rechargeTime, \
                COALESCE(PendingRobotChanges.maxOre, Robot.maxOre) AS maxOre, \
                COALESCE(PendingRobotChanges.miningSpeed, Robot.miningSpeed) AS miningSpeed, \
                COALESCE(PendingRobotChanges.maxTurns, Robot.maxTurns) AS maxTurns, \
                COALESCE(PendingRobotChanges.memorySize, Robot.memorySize) AS memorySize, \
                COALESCE(PendingRobotChanges.cpuSpeed, Robot.cpuSpeed) AS cpuSpeed, \
                COALESCE(PendingRobotChanges.forwardSpeed, Robot.forwardSpeed) AS forwardSpeed, \
                COALESCE(PendingRobotChanges.backwardSpeed, Robot.backwardSpeed) AS backwardSpeed, \
                COALESCE(PendingRobotChanges.rotateSpeed, Robot.rotateSpeed) AS rotateSpeed, \
                COALESCE(PendingRobotChanges.robotSize, Robot.robotSize) AS robotSize, \
                COALESCE(PendingRobotChanges.scanTime, Robot.scanTime) AS scanTime, \
                COALESCE(PendingRobotChanges.scanDistance, Robot.scanDistance) AS scanDistance, \
                PendingRobotChanges.robotId IS NOT NULL AS changePending \
         FROM Robot \
         LEFT JOIN PendingRobotChanges ON PendingRobotChanges.robotId = Robot.id \
         INNER JOIN RobotPart OreContainer \
           ON OreContainer.id = COALESCE(PendingRobotChanges.oreContainerId, Robot.oreContainerId) \
         INNER JOIN RobotPart MiningUnit \
           ON MiningUnit.id = COALESCE(PendingRobotChanges.miningUnitId, Robot.miningUnitId) \
         INNER JOIN RobotPart Battery \
           ON Battery.id = COALESCE(PendingRobotChanges.batteryId, Robot.batteryId) \
         INNER JOIN RobotPart MemoryModule \
           ON MemoryModule.id = COALESCE(PendingRobotChanges.memoryModuleId, Robot.memoryModuleId) \
         INNER JOIN RobotPart Cpu \
           ON Cpu.id = COALESCE(PendingRobotChanges.cpuId, Robot.cpuId) \
         INNER JOIN RobotPart Engine \
           ON Engine.id = COALESCE(PendingRobotChanges.engineId, Robot.engineId) \
         INNER JOIN RobotPart OreScanner \
           ON OreScanner.id = COALESCE(PendingRobotChanges.oreScannerId, Robot.oreScannerId) \
         WHERE Robot.userId = ? \
         ORDER BY Robot.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(robot_config_state_record).collect()
}

pub async fn list_robot_config_part_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotConfigPartAssetStateRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, String, i32, i32, i64)>(
        "SELECT RobotPart.typeId, \
                RobotPart.id, \
                RobotPart.partName, \
                RobotPart.memoryCapacity, \
                UserRobotPartAsset.totalOwned, \
                (SELECT COUNT(*) \
                 FROM Robot \
                 LEFT JOIN PendingRobotChanges ON PendingRobotChanges.robotId = Robot.id \
                 WHERE Robot.userId = ? \
                   AND (Robot.oreContainerId = RobotPart.id \
                        OR Robot.miningUnitId = RobotPart.id \
                        OR Robot.batteryId = RobotPart.id \
                        OR Robot.memoryModuleId = RobotPart.id \
                        OR Robot.cpuId = RobotPart.id \
                        OR Robot.engineId = RobotPart.id \
                        OR Robot.oreScannerId = RobotPart.id \
                        OR PendingRobotChanges.oreContainerId = RobotPart.id \
                        OR PendingRobotChanges.miningUnitId = RobotPart.id \
                        OR PendingRobotChanges.batteryId = RobotPart.id \
                        OR PendingRobotChanges.memoryModuleId = RobotPart.id \
                        OR PendingRobotChanges.cpuId = RobotPart.id \
                        OR PendingRobotChanges.engineId = RobotPart.id \
                        OR PendingRobotChanges.oreScannerId = RobotPart.id)) AS assigned \
         FROM UserRobotPartAsset \
         INNER JOIN RobotPart ON RobotPart.id = UserRobotPartAsset.robotPartId \
         WHERE UserRobotPartAsset.userId = ? \
         ORDER BY RobotPart.typeId, RobotPart.id",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(type_id, robot_part_id, part_name, memory_capacity, total_owned, assigned)| {
                RobotConfigPartAssetStateRecord {
                    type_id,
                    robot_part_id,
                    part_name,
                    memory_capacity,
                    unassigned: total_owned.saturating_sub(assigned as i32),
                }
            },
        )
        .collect())
}

pub async fn get_robot(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, userId, robotName, sourceCode, programSourceId, oreContainerId, \
                miningUnitId, batteryId, memoryModuleId, cpuId, engineId, oreScannerId, \
                rechargeTime, maxOre, miningSpeed, maxTurns, memorySize, cpuSpeed, \
                forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, scanDistance, \
                totalMiningRuns \
         FROM Robot \
         WHERE id = ?",
    )
    .bind(robot_id)
    .fetch_optional(pool)
    .await?;

    row.map(robot_record).transpose()
}

pub async fn get_ai_robot(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<crate::AIRobotRecord>, sqlx::Error> {
    sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            i32,
            i32,
            i32,
            i32,
            f64,
            f64,
            i32,
            f64,
            i32,
            i32,
        ),
    >(
        "SELECT id, robotName, sourceCode, maxOre, miningSpeed, maxTurns, cpuSpeed, \
                forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, scanDistance \
         FROM AIRobot \
         WHERE id = ?",
    )
    .bind(robot_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(
                id,
                robot_name,
                source_code,
                max_ore,
                mining_speed,
                max_turns,
                cpu_speed,
                forward_speed,
                backward_speed,
                rotate_speed,
                robot_size,
                scan_time,
                scan_distance,
            )| crate::AIRobotRecord {
                id,
                robot_name,
                source_code,
                max_ore,
                mining_speed,
                max_turns,
                cpu_speed,
                forward_speed,
                backward_speed,
                rotate_speed,
                robot_size,
                scan_time,
                scan_distance,
            },
        )
    })
}

pub async fn load_robot_stats_header(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotStatsHeaderRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, String, i32)>(
        "SELECT Robot.id, Robot.robotName, User.username, Robot.totalMiningRuns \
         FROM Robot \
         INNER JOIN User ON User.id = Robot.userId \
         WHERE Robot.id = ?",
    )
    .bind(robot_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(robot_id, robot_name, username, total_mining_runs)| RobotStatsHeaderRecord {
                robot_id,
                robot_name,
                username,
                total_mining_runs,
            },
        )
    })
}

pub async fn list_robot_lifetime_ore_stats(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Vec<RobotLifetimeOreStatRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32, i32)>(
        "SELECT Ore.id, Ore.oreName, RobotLifetimeResult.amount, RobotLifetimeResult.tax \
         FROM RobotLifetimeResult \
         INNER JOIN Ore ON Ore.id = RobotLifetimeResult.oreId \
         WHERE RobotLifetimeResult.robotId = ? \
         ORDER BY Ore.id",
    )
    .bind(robot_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(ore_id, ore_name, amount, tax)| RobotLifetimeOreStatRecord {
                    ore_id,
                    ore_name,
                    amount,
                    tax,
                },
            )
            .collect()
    })
}

pub async fn list_robot_mining_area_stats(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Vec<RobotMiningAreaStatRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String, i32, f64)>(
        "SELECT MiningArea.id, MiningArea.areaName, RobotMiningAreaScore.totalRuns, \
                RobotMiningAreaScore.score \
         FROM RobotMiningAreaScore \
         INNER JOIN MiningArea ON MiningArea.id = RobotMiningAreaScore.miningAreaId \
         WHERE RobotMiningAreaScore.robotId = ? \
         ORDER BY MiningArea.id",
    )
    .bind(robot_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(mining_area_id, area_name, total_runs, score)| RobotMiningAreaStatRecord {
                    mining_area_id,
                    area_name,
                    total_runs,
                    score,
                },
            )
            .collect()
    })
}

pub async fn list_robot_mining_area_scores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotMiningAreaScoreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, f64)>(
        "SELECT RobotMiningAreaScore.robotId, RobotMiningAreaScore.miningAreaId, \
                RobotMiningAreaScore.score \
         FROM RobotMiningAreaScore \
         INNER JOIN Robot ON Robot.id = RobotMiningAreaScore.robotId \
         WHERE Robot.userId = ? \
         ORDER BY RobotMiningAreaScore.robotId, RobotMiningAreaScore.miningAreaId",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(robot_id, mining_area_id, score)| RobotMiningAreaScoreRecord {
                    robot_id,
                    mining_area_id,
                    score,
                },
            )
            .collect()
    })
}
pub async fn count_user_robots(pool: &MySqlPool, user_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE userId = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
}
