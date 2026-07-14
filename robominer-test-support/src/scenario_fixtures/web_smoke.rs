use robominer_db::MySqlPool;

use crate::{insert_robot, insert_row_id, unique_prefix};

pub struct WebSmokeDbFixture {
    pub user_id: i64,
    pub robot_name: String,
    pub area_name: String,
    pub mining_queue_id: i64,
    pub robot_id: i64,
    pub mining_area_id: i64,
    ore_id: i64,
    ai_robot_id: i64,
    ore_price_id: i64,
}

impl WebSmokeDbFixture {
    pub async fn create(pool: &MySqlPool, user_id: i64, prefix: &str) -> Self {
        let robot_name = format!("{prefix}-robot");
        let area_name = format!("{prefix}-area");

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
        insert_row_id(
            pool,
            sqlx::query("INSERT INTO OrePriceAmount (orePriceId, oreId, amount) VALUES (?, ?, ?)")
                .bind(ore_price_id)
                .bind(ore_id)
                .bind(1),
        )
        .await;

        insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
                 VALUES (?, ?, ?, 1000)",
            )
            .bind(user_id)
            .bind(ore_id)
            .bind(25),
        )
        .await;

        let ai_robot_id =
            insert_robot(pool, user_id, &format!("{prefix}-ai"), "rotate(90);", 1).await;
        let robot_id = insert_robot(pool, user_id, &robot_name, "mine();", 1).await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
            )
            .bind(&area_name)
            .bind(ore_price_id)
            .bind(ai_robot_id),
        )
        .await;

        sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
            .bind(user_id)
            .bind(mining_area_id)
            .execute(pool)
            .await
            .expect("failed to grant mining area");

        let enqueued = match robominer_domain::enqueue_mining(
            pool,
            robominer_db::EnqueueMiningRequest {
                user_id,
                robot_id,
                mining_area_id,
                fill: false,
            },
        )
        .await
        {
            Ok(Ok(enqueued)) => enqueued,
            Ok(Err(rejection)) => panic!("enqueue mining rejected: {rejection:?}"),
            Err(error) => panic!("enqueue mining failed: {error}"),
        };
        assert_eq!(enqueued.inserted_queues, 1, "expected one queued mining run");

        let mining_queue_id: i64 = sqlx::query_scalar(
            "SELECT id FROM MiningQueue WHERE robotId = ? AND miningAreaId = ? \
             ORDER BY id DESC LIMIT 1",
        )
        .bind(robot_id)
        .bind(mining_area_id)
        .fetch_one(pool)
        .await
        .expect("failed to load inserted mining queue id");

        Self {
            user_id,
            robot_name,
            area_name,
            mining_queue_id,
            ore_id,
            ore_price_id,
            ai_robot_id,
            robot_id,
            mining_area_id,
        }
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotActionsDone WHERE miningQueueId = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserMiningArea WHERE userId = ? AND miningAreaId = ?")
            .bind(self.user_id)
            .bind(self.mining_area_id)
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
            .bind(self.robot_id)
            .bind(self.ai_robot_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
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

pub fn web_smoke_prefix() -> String {
    unique_prefix("rust-web-smoke")
}
