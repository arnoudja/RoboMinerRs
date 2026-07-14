use robominer_db::MySqlPool;
use sqlx::Row;

use crate::{insert_row_id, unique_prefix};

pub struct RobotConfigFixture {
    pub user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub robot_part_type_id: i64,
    pub program_source_id: i64,
    pub robot_id: i64,
    pub mining_area_id: Option<i64>,
    pub mining_queue_id: Option<i64>,
    pub current_part_ids: [i64; 7],
    pub new_part_ids: [i64; 7],
}

impl RobotConfigFixture {
    pub async fn create(
        pool: &MySqlPool,
        queued: bool,
        own_new_parts: bool,
        compiled_size: i32,
    ) -> Self {
        let prefix = unique_prefix("rust-robot-config-cli");

        let ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
        )
        .await;
        let ore_price_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
                .bind(format!("{prefix}-price")),
        )
        .await;
        let robot_part_type_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO RobotPartType (typeName) VALUES (?)")
                .bind(format!("{prefix}-type")),
        )
        .await;

        let mut current_part_ids = [0_i64; 7];
        let mut new_part_ids = [0_i64; 7];
        for index in 0..7 {
            current_part_ids[index] = insert_robot_config_part(
                pool,
                robot_part_type_id,
                ore_id,
                ore_price_id,
                &format!("{prefix}-current-{index}"),
                1,
            )
            .await;
            new_part_ids[index] = insert_robot_config_part(
                pool,
                robot_part_type_id,
                ore_id,
                ore_price_id,
                &format!("{prefix}-new-{index}"),
                2,
            )
            .await;
        }

        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let program_source_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO ProgramSource \
                 (userId, sourceName, sourceCode, verified, compiledSize) \
                 VALUES (?, ?, 'move(2);', true, ?)",
            )
            .bind(user_id)
            .bind(format!("{prefix}-source"))
            .bind(compiled_size),
        )
        .await;

        for robot_part_id in current_part_ids {
            insert_user_robot_part_asset(pool, user_id, robot_part_id, 1).await;
        }
        if own_new_parts {
            for robot_part_id in new_part_ids {
                insert_user_robot_part_asset(pool, user_id, robot_part_id, 1).await;
            }
        }

        let robot_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO Robot \
                 (userId, robotName, sourceCode, programSourceId, oreContainerId, miningUnitId, \
                  batteryId, memoryModuleId, cpuId, engineId, oreScannerId, rechargeTime, maxOre, \
                  miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, \
                  rotateSpeed, robotSize, scanTime, scanDistance, rechargeEndTime, miningEndTime) \
                 VALUES (?, ?, 'move(1);', ?, ?, ?, ?, ?, ?, ?, ?, 6, 6, 6, 6, 6, 6, \
                         1.0, 1.0, 1, 1.0, 0, 0, TIMESTAMPADD(SECOND, -10, NOW()), NULL)",
            )
            .bind(user_id)
            .bind(format!("{prefix}_old"))
            .bind(program_source_id)
            .bind(current_part_ids[0])
            .bind(current_part_ids[1])
            .bind(current_part_ids[2])
            .bind(current_part_ids[3])
            .bind(current_part_ids[4])
            .bind(current_part_ids[5])
            .bind(current_part_ids[6]),
        )
        .await;

        let (mining_area_id, mining_queue_id) = if queued {
            let mining_area_id = insert_row_id(
                pool,
                sqlx::query(
                    "INSERT INTO MiningArea \
                     (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                     VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
                )
                .bind(format!("{prefix}-area"))
                .bind(ore_price_id)
                .bind(robot_id),
            )
            .await;
            let mining_queue_id = insert_row_id(
                pool,
                sqlx::query(
                    "INSERT INTO MiningQueue (miningAreaId, robotId, miningEndTime) \
                     VALUES (?, ?, NULL)",
                )
                .bind(mining_area_id)
                .bind(robot_id),
            )
            .await;
            (Some(mining_area_id), Some(mining_queue_id))
        } else {
            (None, None)
        };

        Self {
            user_id,
            ore_id,
            ore_price_id,
            robot_part_type_id,
            program_source_id,
            robot_id,
            mining_area_id,
            mining_queue_id,
            current_part_ids,
            new_part_ids,
        }
    }

    pub fn update_args(&self, database_url: &str) -> Vec<String> {
        self.update_args_for_parts(database_url, &self.new_part_ids)
    }

    pub fn update_args_for_parts(&self, database_url: &str, part_ids: &[i64; 7]) -> Vec<String> {
        vec![
            "--database-url".to_string(),
            database_url.to_string(),
            "update-robot-config".to_string(),
            "--user-id".to_string(),
            self.user_id.to_string(),
            "--robot-id".to_string(),
            self.robot_id.to_string(),
            "--robot-name".to_string(),
            "rust_bot".to_string(),
            "--program-source-id".to_string(),
            self.program_source_id.to_string(),
            "--ore-container-id".to_string(),
            part_ids[0].to_string(),
            "--mining-unit-id".to_string(),
            part_ids[1].to_string(),
            "--battery-id".to_string(),
            part_ids[2].to_string(),
            "--memory-module-id".to_string(),
            part_ids[3].to_string(),
            "--cpu-id".to_string(),
            part_ids[4].to_string(),
            "--engine-id".to_string(),
            part_ids[5].to_string(),
            "--ore-scanner-id".to_string(),
            part_ids[6].to_string(),
        ]
    }

    pub async fn assert_pending_parts(&self, pool: &MySqlPool, expected_part_ids: &[i64; 7]) {
        let pending_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM PendingRobotChanges WHERE robotId = ?")
                .bind(self.robot_id)
                .fetch_one(pool)
                .await
                .expect("failed to count pending robot changes");
        assert_eq!(pending_count, 1, "expected exactly one pending change row");

        let pending_row = sqlx::query(
            "SELECT sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, cpuId, \
                    engineId, oreScannerId, oldOreContainerId, rechargeTime, maxOre, miningSpeed, \
                    maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
                    scanTime, scanDistance \
             FROM PendingRobotChanges \
             WHERE robotId = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load pending robot changes");

        assert_eq!(pending_row.get::<String, _>("sourceCode"), "move(2);");
        assert_eq!(
            pending_row.get::<i64, _>("oreContainerId"),
            expected_part_ids[0]
        );
        assert_eq!(
            pending_row.get::<i64, _>("miningUnitId"),
            expected_part_ids[1]
        );
        assert_eq!(pending_row.get::<i64, _>("batteryId"), expected_part_ids[2]);
        assert_eq!(
            pending_row.get::<i64, _>("memoryModuleId"),
            expected_part_ids[3]
        );
        assert_eq!(pending_row.get::<i64, _>("cpuId"), expected_part_ids[4]);
        assert_eq!(pending_row.get::<i64, _>("engineId"), expected_part_ids[5]);
        assert_eq!(
            pending_row.get::<i64, _>("oreScannerId"),
            expected_part_ids[6]
        );
        assert_eq!(
            pending_row.get::<i64, _>("oldOreContainerId"),
            self.current_part_ids[0]
        );
        assert_robot_parameters(pending_row);

        let robot_row = sqlx::query(
            "SELECT robotName, sourceCode, oreContainerId \
             FROM Robot \
             WHERE id = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot");
        assert_eq!(robot_row.get::<String, _>("robotName"), "rust_bot");
        assert_eq!(robot_row.get::<String, _>("sourceCode"), "move(1);");
        assert_eq!(
            robot_row.get::<i64, _>("oreContainerId"),
            self.current_part_ids[0]
        );
    }

    pub async fn assert_active_updated(&self, pool: &MySqlPool) {
        let row = sqlx::query(
            "SELECT robotName, sourceCode, oreContainerId, miningUnitId, batteryId, \
                    memoryModuleId, cpuId, engineId, oreScannerId, rechargeTime, maxOre, \
                    miningSpeed, maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, \
                    rotateSpeed, scanTime, scanDistance \
             FROM Robot \
             WHERE id = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load updated robot");

        assert_eq!(row.get::<String, _>("robotName"), "rust_bot");
        assert_eq!(row.get::<String, _>("sourceCode"), "move(2);");
        assert_eq!(row.get::<i64, _>("oreContainerId"), self.new_part_ids[0]);
        assert_eq!(row.get::<i64, _>("miningUnitId"), self.new_part_ids[1]);
        assert_eq!(row.get::<i64, _>("batteryId"), self.new_part_ids[2]);
        assert_eq!(row.get::<i64, _>("memoryModuleId"), self.new_part_ids[3]);
        assert_eq!(row.get::<i64, _>("cpuId"), self.new_part_ids[4]);
        assert_eq!(row.get::<i64, _>("engineId"), self.new_part_ids[5]);
        assert_eq!(row.get::<i64, _>("oreScannerId"), self.new_part_ids[6]);
        assert_robot_parameters(row);
    }

    pub async fn assert_pending_updated(&self, pool: &MySqlPool) {
        let robot_row = sqlx::query(
            "SELECT robotName, sourceCode, oreContainerId \
             FROM Robot \
             WHERE id = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot");
        assert_eq!(robot_row.get::<String, _>("robotName"), "rust_bot");
        assert_eq!(robot_row.get::<String, _>("sourceCode"), "move(1);");
        assert_eq!(
            robot_row.get::<i64, _>("oreContainerId"),
            self.current_part_ids[0]
        );

        let pending_row = sqlx::query(
            "SELECT sourceCode, oreContainerId, miningUnitId, batteryId, memoryModuleId, cpuId, \
                    engineId, oreScannerId, oldOreContainerId, rechargeTime, maxOre, miningSpeed, \
                    maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
                    scanTime, scanDistance \
             FROM PendingRobotChanges \
             WHERE robotId = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load pending robot changes");

        assert_eq!(pending_row.get::<String, _>("sourceCode"), "move(2);");
        assert_eq!(
            pending_row.get::<i64, _>("oreContainerId"),
            self.new_part_ids[0]
        );
        assert_eq!(
            pending_row.get::<i64, _>("miningUnitId"),
            self.new_part_ids[1]
        );
        assert_eq!(pending_row.get::<i64, _>("batteryId"), self.new_part_ids[2]);
        assert_eq!(
            pending_row.get::<i64, _>("memoryModuleId"),
            self.new_part_ids[3]
        );
        assert_eq!(pending_row.get::<i64, _>("cpuId"), self.new_part_ids[4]);
        assert_eq!(pending_row.get::<i64, _>("engineId"), self.new_part_ids[5]);
        assert_eq!(pending_row.get::<i64, _>("oreScannerId"), self.new_part_ids[6]);
        assert_eq!(
            pending_row.get::<i64, _>("oldOreContainerId"),
            self.current_part_ids[0]
        );
        assert_robot_parameters(pending_row);
    }

    pub async fn assert_active_unchanged(&self, pool: &MySqlPool) {
        let row = sqlx::query(
            "SELECT robotName, sourceCode, oreContainerId \
             FROM Robot \
             WHERE id = ?",
        )
        .bind(self.robot_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot");

        assert_ne!(row.get::<String, _>("robotName"), "rust_bot");
        assert_eq!(row.get::<String, _>("sourceCode"), "move(1);");
        assert_eq!(
            row.get::<i64, _>("oreContainerId"),
            self.current_part_ids[0]
        );
        self.assert_no_pending_changes(pool).await;
    }

    pub async fn assert_no_pending_changes(&self, pool: &MySqlPool) {
        let pending_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM PendingRobotChanges WHERE robotId = ?")
                .bind(self.robot_id)
                .fetch_one(pool)
                .await
                .expect("failed to load pending change count");
        assert_eq!(pending_count, 0);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM PendingRobotChanges WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        if let Some(mining_queue_id) = self.mining_queue_id {
            let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
                .bind(mining_queue_id)
                .execute(pool)
                .await;
        }
        if let Some(mining_area_id) = self.mining_area_id {
            let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
                .bind(mining_area_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
            .bind(self.program_source_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        for robot_part_id in self.current_part_ids.into_iter().chain(self.new_part_ids) {
            let _ = sqlx::query("DELETE FROM RobotPart WHERE id = ?")
                .bind(robot_part_id)
                .execute(pool)
                .await;
        }
        let _ = sqlx::query("DELETE FROM RobotPartType WHERE id = ?")
            .bind(self.robot_part_type_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
            .bind(self.ore_price_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(self.ore_id)
            .execute(pool)
            .await;
    }
}

pub async fn insert_robot_config_part(pool: &MySqlPool,
    robot_part_type_id: i64,
    ore_id: i64,
    ore_price_id: i64,
    part_name: &str,
    scale: i32,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO RobotPart \
             (typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
              batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
              rotateCapacity, rechargeTime, weight, volume, powerUsage) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(robot_part_type_id)
        .bind(ore_id)
        .bind(part_name)
        .bind(ore_price_id)
        .bind(2 * scale)
        .bind(3 * scale)
        .bind(10 * scale)
        .bind(4 * scale)
        .bind(5 * scale)
        .bind(6 * scale)
        .bind(3 * scale)
        .bind(2 * scale)
        .bind(scale)
        .bind(2 * scale)
        .bind(8 * scale)
        .bind(scale),
    )
    .await
}

pub async fn insert_user_robot_part_asset(pool: &MySqlPool,
    user_id: i64,
    robot_part_id: i64,
    total_owned: i32,
) {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO UserRobotPartAsset (userId, robotPartId, totalOwned) \
             VALUES (?, ?, ?)",
        )
        .bind(user_id)
        .bind(robot_part_id)
        .bind(total_owned),
    )
    .await;
}

pub fn assert_robot_parameters(row: sqlx::mysql::MySqlRow) {
    assert_eq!(row.get::<i32, _>("rechargeTime"), 14);
    assert_eq!(row.get::<i32, _>("maxOre"), 28);
    assert_eq!(row.get::<i32, _>("miningSpeed"), 42);
    assert_eq!(row.get::<i32, _>("maxTurns"), 10);
    assert_eq!(row.get::<i32, _>("memorySize"), 56);
    assert_eq!(row.get::<i32, _>("cpuSpeed"), 70);
    assert!((row.get::<f64, _>("forwardSpeed") - 9.0).abs() < f64::EPSILON);
    assert!((row.get::<f64, _>("backwardSpeed") - 4.5).abs() < f64::EPSILON);
    assert_eq!(row.get::<i32, _>("rotateSpeed"), 20);
}

