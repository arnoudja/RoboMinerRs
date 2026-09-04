use sqlx::MySqlPool;

use crate::{
    RobotConfigPartAssetStateRecord, RobotConfigStateRecord, RobotLifetimeOreStatRecord,
    RobotMiningAreaScoreRecord, RobotMiningAreaStatRecord, RobotRecord, RobotStatsHeaderRecord,
};

#[derive(sqlx::FromRow)]
struct RobotRow {
    id: i64,
    #[sqlx(rename = "userId")]
    user_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    #[sqlx(rename = "sourceCode")]
    source_code: String,
    #[sqlx(rename = "programSourceId")]
    program_source_id: Option<i64>,
    #[sqlx(rename = "oreContainerId")]
    ore_container_id: Option<i64>,
    #[sqlx(rename = "miningUnitId")]
    mining_unit_id: Option<i64>,
    #[sqlx(rename = "batteryId")]
    battery_id: Option<i64>,
    #[sqlx(rename = "memoryModuleId")]
    memory_module_id: Option<i64>,
    #[sqlx(rename = "cpuId")]
    cpu_id: Option<i64>,
    #[sqlx(rename = "engineId")]
    engine_id: Option<i64>,
    #[sqlx(rename = "oreScannerId")]
    ore_scanner_id: Option<i64>,
    #[sqlx(rename = "rechargeTime")]
    recharge_time: i32,
    #[sqlx(rename = "maxOre")]
    max_ore: i32,
    #[sqlx(rename = "miningSpeed")]
    mining_speed: i32,
    #[sqlx(rename = "maxTurns")]
    max_turns: i32,
    #[sqlx(rename = "memorySize")]
    memory_size: i32,
    #[sqlx(rename = "cpuSpeed")]
    cpu_speed: i32,
    #[sqlx(rename = "forwardSpeed")]
    forward_speed: f64,
    #[sqlx(rename = "backwardSpeed")]
    backward_speed: f64,
    #[sqlx(rename = "rotateSpeed")]
    rotate_speed: i32,
    #[sqlx(rename = "robotSize")]
    robot_size: f64,
    #[sqlx(rename = "scanTime")]
    scan_time: i32,
    #[sqlx(rename = "scanDistance")]
    scan_distance: i32,
    #[sqlx(rename = "totalMiningRuns")]
    total_mining_runs: i32,
}

impl From<RobotRow> for RobotRecord {
    fn from(row: RobotRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            robot_name: row.robot_name,
            source_code: row.source_code,
            program_source_id: row.program_source_id,
            ore_container_id: row.ore_container_id,
            mining_unit_id: row.mining_unit_id,
            battery_id: row.battery_id,
            memory_module_id: row.memory_module_id,
            cpu_id: row.cpu_id,
            engine_id: row.engine_id,
            ore_scanner_id: row.ore_scanner_id,
            recharge_time: row.recharge_time,
            max_ore: row.max_ore,
            mining_speed: row.mining_speed,
            max_turns: row.max_turns,
            memory_size: row.memory_size,
            cpu_speed: row.cpu_speed,
            forward_speed: row.forward_speed,
            backward_speed: row.backward_speed,
            rotate_speed: row.rotate_speed,
            robot_size: row.robot_size,
            scan_time: row.scan_time,
            scan_distance: row.scan_distance,
            total_mining_runs: row.total_mining_runs,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RobotConfigStateRow {
    #[sqlx(rename = "robotId")]
    robot_id: i64,
    #[sqlx(rename = "robotName")]
    robot_name: String,
    #[sqlx(rename = "programSourceId")]
    program_source_id: i64,
    #[sqlx(rename = "oreContainerId")]
    ore_container_id: i64,
    #[sqlx(rename = "oreContainerName")]
    ore_container_name: String,
    #[sqlx(rename = "miningUnitId")]
    mining_unit_id: i64,
    #[sqlx(rename = "miningUnitName")]
    mining_unit_name: String,
    #[sqlx(rename = "batteryId")]
    battery_id: i64,
    #[sqlx(rename = "batteryName")]
    battery_name: String,
    #[sqlx(rename = "batteryCapacity")]
    battery_capacity: i32,
    #[sqlx(rename = "memoryModuleId")]
    memory_module_id: i64,
    #[sqlx(rename = "memoryModuleName")]
    memory_module_name: String,
    #[sqlx(rename = "cpuId")]
    cpu_id: i64,
    #[sqlx(rename = "cpuName")]
    cpu_name: String,
    #[sqlx(rename = "engineId")]
    engine_id: i64,
    #[sqlx(rename = "engineName")]
    engine_name: String,
    #[sqlx(rename = "engineForwardCapacity")]
    engine_forward_capacity: i32,
    #[sqlx(rename = "oreScannerId")]
    ore_scanner_id: i64,
    #[sqlx(rename = "oreScannerName")]
    ore_scanner_name: String,
    #[sqlx(rename = "rechargeTime")]
    recharge_time: i32,
    #[sqlx(rename = "maxOre")]
    max_ore: i32,
    #[sqlx(rename = "miningSpeed")]
    mining_speed: i32,
    #[sqlx(rename = "maxTurns")]
    max_turns: i32,
    #[sqlx(rename = "memorySize")]
    memory_size: i32,
    #[sqlx(rename = "cpuSpeed")]
    cpu_speed: i32,
    #[sqlx(rename = "forwardSpeed")]
    forward_speed: f64,
    #[sqlx(rename = "backwardSpeed")]
    backward_speed: f64,
    #[sqlx(rename = "rotateSpeed")]
    rotate_speed: i32,
    #[sqlx(rename = "robotSize")]
    robot_size: f64,
    #[sqlx(rename = "scanTime")]
    scan_time: i32,
    #[sqlx(rename = "scanDistance")]
    scan_distance: i32,
    #[sqlx(rename = "changePending")]
    change_pending: bool,
}

impl From<RobotConfigStateRow> for RobotConfigStateRecord {
    fn from(row: RobotConfigStateRow) -> Self {
        Self {
            robot_id: row.robot_id,
            robot_name: row.robot_name,
            program_source_id: row.program_source_id,
            ore_container_id: row.ore_container_id,
            ore_container_name: row.ore_container_name,
            mining_unit_id: row.mining_unit_id,
            mining_unit_name: row.mining_unit_name,
            battery_id: row.battery_id,
            battery_name: row.battery_name,
            battery_capacity: row.battery_capacity,
            memory_module_id: row.memory_module_id,
            memory_module_name: row.memory_module_name,
            cpu_id: row.cpu_id,
            cpu_name: row.cpu_name,
            engine_id: row.engine_id,
            engine_name: row.engine_name,
            engine_forward_capacity: row.engine_forward_capacity,
            ore_scanner_id: row.ore_scanner_id,
            ore_scanner_name: row.ore_scanner_name,
            recharge_time: row.recharge_time,
            max_ore: row.max_ore,
            mining_speed: row.mining_speed,
            max_turns: row.max_turns,
            memory_size: row.memory_size,
            cpu_speed: row.cpu_speed,
            forward_speed: row.forward_speed,
            backward_speed: row.backward_speed,
            rotate_speed: row.rotate_speed,
            robot_size: row.robot_size,
            scan_time: row.scan_time,
            scan_distance: row.scan_distance,
            change_pending: row.change_pending,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RobotConfigPartAssetRow {
    #[sqlx(rename = "typeId")]
    type_id: i64,
    #[sqlx(rename = "id")]
    robot_part_id: i64,
    #[sqlx(rename = "partName")]
    part_name: String,
    #[sqlx(rename = "oreCapacity")]
    ore_capacity: i32,
    #[sqlx(rename = "miningCapacity")]
    mining_capacity: i32,
    #[sqlx(rename = "batteryCapacity")]
    battery_capacity: i32,
    #[sqlx(rename = "memoryCapacity")]
    memory_capacity: i32,
    #[sqlx(rename = "cpuCapacity")]
    cpu_capacity: i32,
    #[sqlx(rename = "forwardCapacity")]
    forward_capacity: i32,
    #[sqlx(rename = "scanDistance")]
    scan_distance: i32,
    #[sqlx(rename = "totalOwned")]
    total_owned: i32,
    assigned: i64,
}

impl From<RobotConfigPartAssetRow> for RobotConfigPartAssetStateRecord {
    fn from(row: RobotConfigPartAssetRow) -> Self {
        Self {
            type_id: row.type_id,
            robot_part_id: row.robot_part_id,
            part_name: row.part_name,
            ore_capacity: row.ore_capacity,
            mining_capacity: row.mining_capacity,
            battery_capacity: row.battery_capacity,
            memory_capacity: row.memory_capacity,
            cpu_capacity: row.cpu_capacity,
            forward_capacity: row.forward_capacity,
            scan_distance: row.scan_distance,
            unassigned: row.total_owned.saturating_sub(row.assigned as i32),
        }
    }
}

pub async fn list_robot_config_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotConfigStateRecord>, sqlx::Error> {
    crate::reconcile_pending_robot_changes_for_user(pool, user_id).await?;

    sqlx::query_as::<_, RobotConfigStateRow>(
        "SELECT Robot.id AS robotId, \
                Robot.robotName, \
                Robot.programSourceId, \
                COALESCE(PendingRobotChanges.oreContainerId, Robot.oreContainerId) AS oreContainerId, \
                OreContainer.partName AS oreContainerName, \
                COALESCE(PendingRobotChanges.miningUnitId, Robot.miningUnitId) AS miningUnitId, \
                MiningUnit.partName AS miningUnitName, \
                COALESCE(PendingRobotChanges.batteryId, Robot.batteryId) AS batteryId, \
                Battery.partName AS batteryName, \
                Battery.batteryCapacity AS batteryCapacity, \
                COALESCE(PendingRobotChanges.memoryModuleId, Robot.memoryModuleId) AS memoryModuleId, \
                MemoryModule.partName AS memoryModuleName, \
                COALESCE(PendingRobotChanges.cpuId, Robot.cpuId) AS cpuId, \
                Cpu.partName AS cpuName, \
                COALESCE(PendingRobotChanges.engineId, Robot.engineId) AS engineId, \
                Engine.partName AS engineName, \
                Engine.forwardCapacity AS engineForwardCapacity, \
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
    .await
    .map(|rows| {
        rows.into_iter()
            .map(RobotConfigStateRecord::from)
            .collect()
    })
}

pub async fn list_robot_config_part_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<RobotConfigPartAssetStateRecord>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RobotConfigPartAssetRow>(
        "SELECT RobotPart.typeId, \
                RobotPart.id, \
                RobotPart.partName, \
                RobotPart.oreCapacity, \
                RobotPart.miningCapacity, \
                RobotPart.batteryCapacity, \
                RobotPart.memoryCapacity, \
                RobotPart.cpuCapacity, \
                RobotPart.forwardCapacity, \
                RobotPart.scanDistance, \
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
        .map(RobotConfigPartAssetStateRecord::from)
        .collect())
}

pub async fn get_robot(
    pool: &MySqlPool,
    robot_id: i64,
) -> Result<Option<RobotRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotRow>(
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
    .await
    .map(|row| row.map(RobotRecord::from))
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
            i32,
        ),
    >(
        "SELECT id, robotName, sourceCode, maxOre, miningSpeed, maxTurns, cpuSpeed, \
                forwardSpeed, backwardSpeed, rotateSpeed, robotSize, scanTime, scanDistance, \
                depotSize \
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
                depot_size,
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
                depot_size,
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
