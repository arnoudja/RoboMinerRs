use robominer_db::MySqlPool;
use sqlx::Row;

use crate::{insert_cli_robot, insert_row_id, unique_prefix};

pub struct ClaimResultsFixture {
    pub user_id: i64,
    pub primary_ore_id: i64,
    pub secondary_ore_id: i64,
    pub ore_price_id: i64,
    pub ai_robot_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub mining_queue_id: i64,
}

impl ClaimResultsFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        Self::create_with_mining_end_time(pool, "TIMESTAMPADD(SECOND, -10, NOW())").await
    }

    pub async fn create_with_mining_end_time(pool: &MySqlPool, mining_end_time_sql: &str) -> Self {
        let prefix = unique_prefix("rust-claim-cli");

        let primary_ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)")
                .bind(format!("{prefix}-primary-ore")),
        )
        .await;
        let secondary_ore_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO Ore (oreName) VALUES (?)")
                .bind(format!("{prefix}-secondary-ore")),
        )
        .await;
        let ore_price_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
                .bind(format!("{prefix}-price")),
        )
        .await;
        let user_id = insert_row_id(
            pool,
            sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
                .bind(format!("{prefix}-user"))
                .bind(format!("{prefix}@example.invalid"))
                .bind("test-password"),
        )
        .await;
        let ai_robot_id =
            insert_cli_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);").await;
        let robot_id = insert_cli_robot(pool, user_id, &format!("{prefix}-robot"), "mine();").await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 1, 1, 25, ?)",
            )
            .bind(format!("{prefix}-area"))
            .bind(ore_price_id)
            .bind(ai_robot_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
                 VALUES (?, ?, 10, 2)",
            )
            .bind(mining_area_id)
            .bind(primary_ore_id),
        )
        .await;
        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
                 VALUES (?, ?, 3, 1)",
            )
            .bind(mining_area_id)
            .bind(secondary_ore_id),
        )
        .await;

        let mining_queue_id = insert_row_id(
            pool,
            sqlx::query(&format!(
                "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime, claimed) \
                 VALUES (?, ?, TIMESTAMPADD(SECOND, -20, NOW()), {mining_end_time_sql}, false)"
            ))
            .bind(mining_area_id)
            .bind(robot_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, tax) \
                 VALUES (?, ?, 10, NULL)",
            )
            .bind(mining_queue_id)
            .bind(primary_ore_id),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, 2, 8)",
            )
            .bind(user_id)
            .bind(primary_ore_id),
        )
        .await;

        sqlx::query(
            "INSERT INTO PendingRobotChanges \
             (robotId, sourceCode, rechargeTime, maxOre, miningSpeed, maxTurns, memorySize, \
              cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize, changesCommitTime) \
             VALUES (?, 'mine();', 1, 100, 4, 1, 128, 1, 1.0, 1.0, 90, 1.0, \
                     TIMESTAMPADD(SECOND, -1, NOW()))",
        )
        .bind(robot_id)
        .execute(pool)
        .await
        .expect("failed to insert committed pending robot changes");

        Self {
            user_id,
            primary_ore_id,
            secondary_ore_id,
            ore_price_id,
            ai_robot_id,
            robot_id,
            mining_area_id,
            mining_queue_id,
        }
    }

    pub async fn assert_claimed(&self, pool: &MySqlPool) {
        let claimed: i8 = sqlx::query_scalar("SELECT claimed FROM MiningQueue WHERE id = ?")
            .bind(self.mining_queue_id)
            .fetch_one(pool)
            .await
            .expect("failed to load claimed queue state");
        assert_eq!(claimed, 1);

        let tax: Option<i32> =
            sqlx::query_scalar("SELECT tax FROM MiningOreResult WHERE miningQueueId = ?")
                .bind(self.mining_queue_id)
                .fetch_one(pool)
                .await
                .expect("failed to load mining ore tax");
        assert_eq!(tax, Some(2));

        let user_ore_amount: i32 =
            sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
                .bind(self.user_id)
                .bind(self.primary_ore_id)
                .fetch_one(pool)
                .await
                .expect("failed to load user ore asset");
        assert_eq!(user_ore_amount, 8);

        let total_runs: i32 = sqlx::query_scalar("SELECT totalMiningRuns FROM Robot WHERE id = ?")
            .bind(self.robot_id)
            .fetch_one(pool)
            .await
            .expect("failed to load robot total mining runs");
        assert_eq!(total_runs, 1);

        let robot_lifetime = sqlx::query(
            "SELECT amount, tax FROM RobotLifetimeResult WHERE robotId = ? AND oreId = ?",
        )
        .bind(self.robot_id)
        .bind(self.primary_ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot lifetime result");
        let robot_amount: i32 = robot_lifetime.try_get("amount").unwrap();
        let robot_tax: i32 = robot_lifetime.try_get("tax").unwrap();
        assert_eq!(robot_amount, 10);
        assert_eq!(robot_tax, 2);

        let primary_area_total = sqlx::query(
            "SELECT totalAmount, totalContainerSize \
             FROM MiningAreaLifetimeResult \
             WHERE miningAreaId = ? AND oreId = ?",
        )
        .bind(self.mining_area_id)
        .bind(self.primary_ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load primary mining area lifetime result");
        let primary_amount: i64 = primary_area_total.try_get("totalAmount").unwrap();
        let primary_container: i64 = primary_area_total.try_get("totalContainerSize").unwrap();
        assert_eq!(primary_amount, 10);
        assert_eq!(primary_container, 100);

        let secondary_area_total = sqlx::query(
            "SELECT totalAmount, totalContainerSize \
             FROM MiningAreaLifetimeResult \
             WHERE miningAreaId = ? AND oreId = ?",
        )
        .bind(self.mining_area_id)
        .bind(self.secondary_ore_id)
        .fetch_one(pool)
        .await
        .expect("failed to load secondary mining area lifetime result");
        let secondary_amount: i64 = secondary_area_total.try_get("totalAmount").unwrap();
        let secondary_container: i64 = secondary_area_total.try_get("totalContainerSize").unwrap();
        assert_eq!(secondary_amount, 0);
        assert_eq!(secondary_container, 100);

        let pending_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM PendingRobotChanges WHERE robotId = ?")
                .bind(self.robot_id)
                .fetch_one(pool)
                .await
                .expect("failed to load pending robot changes");
        assert_eq!(pending_count, 0);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM PendingRobotChanges WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotLifetimeResult WHERE robotId = ?")
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningAreaLifetimeResult WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
            .bind(self.mining_area_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Robot WHERE id IN (?, ?)")
            .bind(self.ai_robot_id)
            .bind(self.robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM User WHERE id = ?")
            .bind(self.user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
            .bind(self.ore_price_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM Ore WHERE id IN (?, ?)")
            .bind(self.primary_ore_id)
            .bind(self.secondary_ore_id)
            .execute(pool)
            .await;
    }
}
