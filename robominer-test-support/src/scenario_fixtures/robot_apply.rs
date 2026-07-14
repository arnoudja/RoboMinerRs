use std::collections::HashMap;

use robominer_db::MySqlPool;

use super::default_robot_parts::ensure_default_robot_parts;
use crate::{insert_row_id, unique_prefix};

pub struct RobotApplyFixture {
    pub user_id: i64,
    pub username: String,
    pub password: String,
    pub robot_id: i64,
    pub robot_name: String,
    pub program_source_id: i64,
    pub mining_unit_id: i64,
    pub battery_id: i64,
    pub memory_module_id: i64,
    pub cpu_id: i64,
    pub engine_id: i64,
    pub ore_scanner_id: i64,
    pub spare_ore_container_id: i64,
    pub spare_ore_container_name: String,
    ore_id: i64,
    ore_price_id: i64,
}

impl RobotApplyFixture {
    pub async fn create(
        pool: &MySqlPool,
        user_id: i64,
        username: String,
        password: String,
    ) -> Self {
        ensure_default_robot_parts(pool).await;

        let prefix = unique_prefix("rust-web-robot-apply");

        let robot_row = sqlx::query_as::<_, (i64, String, i64, i64, i64, i64, i64, i64, i64)>(
            "SELECT id, robotName, programSourceId, miningUnitId, batteryId, memoryModuleId, \
                    cpuId, engineId, oreScannerId \
             FROM Robot \
             WHERE userId = ? \
             ORDER BY id \
             LIMIT 1",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("failed to load created robot");

        let (
            robot_id,
            robot_name,
            program_source_id,
            mining_unit_id,
            battery_id,
            memory_module_id,
            cpu_id,
            engine_id,
            ore_scanner_id,
        ) = robot_row;

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
        let spare_ore_container_name = format!("{prefix}-spare-container");
        let spare_ore_container_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO RobotPart \
                 (typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                  batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, \
                  backwardCapacity, rotateCapacity, rechargeTime, scanTime, scanDistance, \
                  weight, volume, powerUsage) \
                 VALUES (1, ?, ?, ?, 20, 1, 100, 8, 2, 50, 50, 50, 1, 0, 0, 10, 10, 1)",
            )
            .bind(ore_id)
            .bind(&spare_ore_container_name)
            .bind(ore_price_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserRobotPartAsset (userId, robotPartId, totalOwned) \
                 VALUES (?, ?, 1)",
            )
            .bind(user_id)
            .bind(spare_ore_container_id),
        )
        .await;

        Self {
            user_id,
            username,
            password,
            robot_id,
            robot_name,
            program_source_id,
            mining_unit_id,
            battery_id,
            memory_module_id,
            cpu_id,
            engine_id,
            ore_scanner_id,
            spare_ore_container_id,
            spare_ore_container_name,
            ore_id,
            ore_price_id,
        }
    }

    pub fn apply_form(&self) -> HashMap<String, String> {
        let robot_id = self.robot_id;
        HashMap::from([
            (
                format!("robotName{robot_id}"),
                valid_apply_robot_name(&self.robot_name),
            ),
            (
                format!("programSourceId{robot_id}"),
                self.program_source_id.to_string(),
            ),
            (
                format!("oreContainerId{robot_id}"),
                self.spare_ore_container_id.to_string(),
            ),
            (
                format!("miningUnitId{robot_id}"),
                self.mining_unit_id.to_string(),
            ),
            (format!("batteryId{robot_id}"), self.battery_id.to_string()),
            (
                format!("memoryModuleId{robot_id}"),
                self.memory_module_id.to_string(),
            ),
            (format!("cpuId{robot_id}"), self.cpu_id.to_string()),
            (format!("engineId{robot_id}"), self.engine_id.to_string()),
            (
                format!("oreScannerId{robot_id}"),
                self.ore_scanner_id.to_string(),
            ),
            ("robotId".to_string(), self.robot_id.to_string()),
        ])
    }

    pub async fn assert_ore_container_id(&self, pool: &MySqlPool, expected: i64) {
        let ore_container_id: i64 =
            sqlx::query_scalar("SELECT oreContainerId FROM Robot WHERE id = ?")
                .bind(self.robot_id)
                .fetch_one(pool)
                .await
                .expect("failed to load robot ore container");
        assert_eq!(ore_container_id, expected);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotPart WHERE id = ?")
            .bind(self.spare_ore_container_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM OrePriceAmount WHERE orePriceId = ?")
            .bind(self.ore_price_id)
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

fn valid_apply_robot_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .take(15)
        .collect();
    if sanitized.is_empty() {
        "robot_one".to_string()
    } else {
        sanitized
    }
}
