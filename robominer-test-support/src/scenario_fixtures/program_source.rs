use robominer_db::MySqlPool;

use super::robot_config::insert_robot_config_part;
use crate::{insert_row_id, unique_prefix};

pub struct ProgramSourceFixture {
    pub user_id: i64,
    pub other_user_id: i64,
    pub program_source_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub robot_part_type_id: i64,
    pub robot_part_ids: [i64; 6],
    pub robot_ids: std::cell::RefCell<Vec<i64>>,
    pub mining_area_id: std::cell::Cell<Option<i64>>,
    pub mining_queue_id: std::cell::Cell<Option<i64>>,
}

impl ProgramSourceFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("rust-source-cli");

        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let other_user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-other-user"))
                .bind(format!("{prefix}-other@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let program_source_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO ProgramSource \
                 (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
                 VALUES (?, ?, 'move(1);', true, 1, '')",
            )
            .bind(user_id)
            .bind(format!("{prefix}-source")),
        )
        .await;

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
        let mut robot_part_ids = [0_i64; 6];
        for (index, robot_part_id) in robot_part_ids.iter_mut().enumerate() {
            *robot_part_id = insert_robot_config_part(
                pool,
                robot_part_type_id,
                ore_id,
                ore_price_id,
                &format!("{prefix}-part-{index}"),
                1,
            )
            .await;
        }

        Self {
            user_id,
            other_user_id,
            program_source_id,
            ore_id,
            ore_price_id,
            robot_part_type_id,
            robot_part_ids,
            robot_ids: std::cell::RefCell::new(Vec::new()),
            mining_area_id: std::cell::Cell::new(None),
            mining_queue_id: std::cell::Cell::new(None),
        }
    }

    pub async fn create_linked_robots(&self, pool: &MySqlPool) {
        let idle_robot_id = self
            .insert_linked_robot(pool, "idle", 128, "move(1);")
            .await;
        let busy_robot_id = self
            .insert_linked_robot(pool, "busy", 128, "move(1);")
            .await;
        let low_memory_robot_id = self
            .insert_linked_robot(pool, "low_memory", 0, "move(1);")
            .await;

        self.robot_ids
            .borrow_mut()
            .extend([idle_robot_id, busy_robot_id, low_memory_robot_id]);

        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
            )
            .bind(format!("rust-source-area-{}", busy_robot_id))
            .bind(self.ore_price_id)
            .bind(busy_robot_id),
        )
        .await;
        let mining_queue_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningQueue (miningAreaId, robotId, miningEndTime) \
                 VALUES (?, ?, NULL)",
            )
            .bind(mining_area_id)
            .bind(busy_robot_id),
        )
        .await;
        self.mining_area_id.set(Some(mining_area_id));
        self.mining_queue_id.set(Some(mining_queue_id));
    }

    pub async fn insert_linked_robot(
        &self,
        pool: &MySqlPool,
        name_suffix: &str,
        memory_size: i32,
        source_code: &str,
    ) -> i64 {
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO Robot \
                 (userId, robotName, sourceCode, programSourceId, oreContainerId, miningUnitId, \
                  batteryId, memoryModuleId, cpuId, engineId, rechargeTime, maxOre, miningSpeed, \
                  maxTurns, memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, \
                  robotSize, rechargeEndTime, miningEndTime) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 1, 1, 1, ?, 1, \
                         1.0, 1.0, 1, 1.0, TIMESTAMPADD(SECOND, -10, NOW()), NULL)",
            )
            .bind(self.user_id)
            .bind(format!("source_{name_suffix}"))
            .bind(source_code)
            .bind(self.program_source_id)
            .bind(self.robot_part_ids[0])
            .bind(self.robot_part_ids[1])
            .bind(self.robot_part_ids[2])
            .bind(self.robot_part_ids[3])
            .bind(self.robot_part_ids[4])
            .bind(self.robot_part_ids[5])
            .bind(memory_size),
        )
        .await
    }

    pub async fn assert_source_updated_and_applied(&self, pool: &MySqlPool) {
        let (source_name, source_code, verified): (String, String, bool) = sqlx::query_as(
            "SELECT sourceName, sourceCode, verified FROM ProgramSource WHERE id = ?",
        )
        .bind(self.program_source_id)
        .fetch_one(pool)
        .await
        .expect("failed to load program source");
        assert_eq!(source_name, "updated source");
        assert_eq!(source_code, "move(2);");
        assert!(verified);

        let robot_ids = self.robot_ids.borrow().clone();
        let idle_source: String = sqlx::query_scalar("SELECT sourceCode FROM Robot WHERE id = ?")
            .bind(robot_ids[0])
            .fetch_one(pool)
            .await
            .expect("failed to load idle robot source");
        let busy_source: String = sqlx::query_scalar("SELECT sourceCode FROM Robot WHERE id = ?")
            .bind(robot_ids[1])
            .fetch_one(pool)
            .await
            .expect("failed to load busy robot source");
        let low_memory_source: String =
            sqlx::query_scalar("SELECT sourceCode FROM Robot WHERE id = ?")
                .bind(robot_ids[2])
                .fetch_one(pool)
                .await
                .expect("failed to load low memory robot source");

        assert_eq!(idle_source, "move(2);");
        assert_eq!(busy_source, "move(1);");
        assert_eq!(low_memory_source, "move(1);");

        let busy_pending_source: String =
            sqlx::query_scalar("SELECT sourceCode FROM PendingRobotChanges WHERE robotId = ?")
                .bind(robot_ids[1])
                .fetch_one(pool)
                .await
                .expect("busy robot should have pending source update");
        assert_eq!(busy_pending_source, "move(2);");
    }

    pub async fn assert_source_unchanged(&self, pool: &MySqlPool) {
        let (source_name, source_code): (String, String) =
            sqlx::query_as("SELECT sourceName, sourceCode FROM ProgramSource WHERE id = ?")
                .bind(self.program_source_id)
                .fetch_one(pool)
                .await
                .expect("failed to load program source");
        assert!(source_name.contains("source"));
        assert_eq!(source_code, "move(1);");
    }

    pub async fn assert_source_deleted(&self, pool: &MySqlPool) {
        let source_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ProgramSource WHERE id = ?")
                .bind(self.program_source_id)
                .fetch_one(pool)
                .await
                .expect("failed to count program sources");
        assert_eq!(source_count, 0);
    }

    pub async fn cleanup_extra_program_source(&self, pool: &MySqlPool, program_source_id: i64) {
        let _ = sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
            .bind(program_source_id)
            .execute(pool)
            .await;
    }

    pub async fn cleanup_without_source(&self, pool: &MySqlPool) {
        self.cleanup_rows(pool, false).await;
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        self.cleanup_rows(pool, true).await;
    }

    pub async fn cleanup_rows(&self, pool: &MySqlPool, delete_source: bool) {
        if let Some(mining_queue_id) = self.mining_queue_id.get() {
            let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
                .bind(mining_queue_id)
                .execute(pool)
                .await;
        }
        if let Some(mining_area_id) = self.mining_area_id.get() {
            let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
                .bind(mining_area_id)
                .execute(pool)
                .await;
        }
        let robot_ids = self.robot_ids.borrow().clone();
        for robot_id in robot_ids {
            let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
                .bind(robot_id)
                .execute(pool)
                .await;
        }
        if delete_source {
            let _ = sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
                .bind(self.program_source_id)
                .execute(pool)
                .await;
        }
        for robot_part_id in self.robot_part_ids {
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
        let _ = sqlx::query("DELETE FROM User WHERE id IN (?, ?)")
            .bind(self.user_id)
            .bind(self.other_user_id)
            .execute(pool)
            .await;
    }
}

pub fn parse_created_program_source_id(stdout: &str) -> i64 {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("Created program source "))
        .expect("created program source line missing")
        .parse()
        .expect("created program source id should parse")
}
