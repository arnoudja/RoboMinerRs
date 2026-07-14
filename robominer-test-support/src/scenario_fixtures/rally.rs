use robominer_db::MySqlPool;
use sqlx::Row;

use crate::{insert_cli_robot, insert_row_id, unique_prefix};

pub struct RallyFixture {
    pub user_id: i64,
    pub ore_id: i64,
    pub ore_price_id: i64,
    pub ai_robot_id: i64,
    pub queued_robot_id: i64,
    pub mining_area_id: i64,
    pub mining_queue_id: i64,
}

impl RallyFixture {
    pub async fn create(pool: &MySqlPool) -> Self {
        let prefix = unique_prefix("rust-rally-cli");

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
        let queued_robot_id = insert_cli_robot(
            pool,
            user_id,
            &format!("{prefix}-queued"),
            "move(1.5); while (mine());",
        )
        .await;
        let mining_area_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningArea \
                 (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
                 VALUES (?, ?, 4, 4, 10, 1, 0, ?)",
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
            .bind(ore_id),
        )
        .await;

        let mining_queue_id = insert_row_id(
            pool,
            sqlx::query(
                "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime) \
                 VALUES (?, ?, NOW(), NULL)",
            )
            .bind(mining_area_id)
            .bind(queued_robot_id),
        )
        .await;

        Self {
            user_id,
            ore_id,
            ore_price_id,
            ai_robot_id,
            queued_robot_id,
            mining_area_id,
            mining_queue_id,
        }
    }

    pub async fn assert_persisted(&self, pool: &MySqlPool) {
        let queue = sqlx::query(
            "SELECT rallyResultId, playerNumber, score, miningEndTime IS NOT NULL AS ended \
             FROM MiningQueue \
             WHERE id = ?",
        )
        .bind(self.mining_queue_id)
        .fetch_one(pool)
        .await
        .expect("failed to load persisted queue row");

        let rally_result_id: Option<i64> = queue.try_get("rallyResultId").unwrap();
        let rally_result_id = rally_result_id.expect("queue should reference rally result");
        let player_number: Option<i32> = queue.try_get("playerNumber").unwrap();
        let score: Option<f64> = queue.try_get("score").unwrap();
        let ended: i8 = queue.try_get("ended").unwrap();

        assert_eq!(player_number, Some(0));
        assert!(score.unwrap_or_default() > 0.0);
        assert_eq!(ended, 1);

        let result_data: String =
            sqlx::query_scalar("SELECT resultData FROM RallyResult WHERE id = ?")
                .bind(rally_result_id)
                .fetch_one(pool)
                .await
                .expect("failed to load rally result data");
        assert!(result_data.contains("var myRobots = {robot: ["));
        assert!(result_data.contains("var myGround = {"));
        assert!(result_data.contains("var myOreTypes = {A:{id:"));

        let ore_amount: Option<i32> = sqlx::query_scalar(
            "SELECT amount FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?",
        )
        .bind(self.mining_queue_id)
        .bind(self.ore_id)
        .fetch_optional(pool)
        .await
        .expect("failed to load ore result");
        assert!(ore_amount.unwrap_or_default() > 0);

        let mine_actions: Option<i32> = sqlx::query_scalar(
            "SELECT amount FROM RobotActionsDone WHERE miningQueueId = ? AND actionType = 6",
        )
        .bind(self.mining_queue_id)
        .fetch_optional(pool)
        .await
        .expect("failed to load action result");
        assert!(mine_actions.unwrap_or_default() >= 1);

        let score_row = sqlx::query(
            "SELECT totalRuns, score FROM RobotMiningAreaScore WHERE robotId = ? AND miningAreaId = ?",
        )
        .bind(self.queued_robot_id)
        .bind(self.mining_area_id)
        .fetch_one(pool)
        .await
        .expect("failed to load robot score");
        let total_runs: i32 = score_row.try_get("totalRuns").unwrap();
        let smoothed_score: f64 = score_row.try_get("score").unwrap();
        assert_eq!(total_runs, 1);
        assert!(smoothed_score > 0.0);
    }

    pub async fn cleanup(&self, pool: &MySqlPool) {
        let rally_result_id: Option<i64> =
            sqlx::query_scalar("SELECT rallyResultId FROM MiningQueue WHERE id = ?")
                .bind(self.mining_queue_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotActionsDone WHERE miningQueueId = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        let _ =
            sqlx::query("DELETE FROM RobotMiningAreaScore WHERE robotId = ? AND miningAreaId = ?")
                .bind(self.queued_robot_id)
                .bind(self.mining_area_id)
                .execute(pool)
                .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(self.mining_queue_id)
            .execute(pool)
            .await;
        if let Some(rally_result_id) = rally_result_id {
            let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
                .bind(rally_result_id)
                .execute(pool)
                .await;
        }
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
            .bind(self.queued_robot_id)
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
        let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
            .bind(self.ore_id)
            .execute(pool)
            .await;
    }
}
