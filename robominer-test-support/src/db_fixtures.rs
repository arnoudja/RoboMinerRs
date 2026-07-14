use std::time::{SystemTime, UNIX_EPOCH};

use robominer_db::MySqlPool;

pub fn unique_prefix(prefix: &str) -> String {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    format!("{prefix}-{unique}")
}

pub async fn insert_row_id<'q>(
    pool: &MySqlPool,
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
) -> i64 {
    query
        .execute(pool)
        .await
        .expect("failed to insert fixture row")
        .last_insert_id() as i64
}

pub async fn insert_ore(pool: &MySqlPool, name: &str) -> i64 {
    insert_row_id(
        pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(name),
    )
    .await
}

pub async fn insert_ore_price(pool: &MySqlPool, description: &str) -> i64 {
    insert_row_id(
        pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)").bind(description),
    )
    .await
}

pub async fn insert_user_with_credentials(
    pool: &MySqlPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query("INSERT INTO User (username, email, password) VALUES (?, ?, ?)")
            .bind(username)
            .bind(email)
            .bind(password_hash),
    )
    .await
}

pub async fn insert_user(pool: &MySqlPool, prefix: &str) -> i64 {
    insert_user_with_credentials(
        pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await
}

pub async fn insert_cli_robot(pool: &MySqlPool, user_id: i64, name: &str, source: &str) -> i64 {
    insert_robot(pool, user_id, name, source, 600).await
}

pub async fn insert_robot(
    pool: &MySqlPool,
    user_id: i64,
    name: &str,
    source: &str,
    max_turns: i32,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO Robot \
             (userId, robotName, sourceCode, rechargeTime, maxOre, miningSpeed, maxTurns, \
              memorySize, cpuSpeed, forwardSpeed, backwardSpeed, rotateSpeed, robotSize, \
              rechargeEndTime, miningEndTime) \
             VALUES (?, ?, ?, 1, 100, 4, ?, 128, 1, 1.0, 1.0, 90, 1.0, \
                     TIMESTAMPADD(SECOND, -10, NOW()), NULL)",
        )
        .bind(user_id)
        .bind(name)
        .bind(source)
        .bind(max_turns),
    )
    .await
}

pub async fn insert_mining_area(
    pool: &MySqlPool,
    prefix: &str,
    ore_price_id: i64,
    ai_robot_id: i64,
    tax_rate: i32,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, ?, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(tax_rate)
        .bind(ai_robot_id),
    )
    .await
}

pub async fn insert_area_supply(
    pool: &MySqlPool,
    mining_area_id: i64,
    ore_id: i64,
    supply: i32,
    radius: i32,
) {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(mining_area_id)
        .bind(ore_id)
        .bind(supply)
        .bind(radius),
    )
    .await;
}

pub async fn insert_finished_queue(
    pool: &MySqlPool,
    mining_area_id: i64,
    robot_id: i64,
    creation_offset_seconds: i32,
    mining_end_offset_seconds: i32,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime, claimed) \
             VALUES (?, ?, TIMESTAMPADD(SECOND, ?, NOW()), TIMESTAMPADD(SECOND, ?, NOW()), false)",
        )
        .bind(mining_area_id)
        .bind(robot_id)
        .bind(creation_offset_seconds)
        .bind(mining_end_offset_seconds),
    )
    .await
}

pub async fn insert_unfinished_queue(pool: &MySqlPool, mining_area_id: i64, robot_id: i64) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime, claimed) \
             VALUES (?, ?, TIMESTAMPADD(SECOND, -5, NOW()), TIMESTAMPADD(SECOND, 60, NOW()), false)",
        )
        .bind(mining_area_id)
        .bind(robot_id),
    )
    .await
}

pub async fn insert_ore_result(pool: &MySqlPool, mining_queue_id: i64, ore_id: i64, amount: i32) {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, tax) \
             VALUES (?, ?, ?, NULL)",
        )
        .bind(mining_queue_id)
        .bind(ore_id)
        .bind(amount),
    )
    .await;
}

pub async fn insert_user_ore_asset(
    pool: &MySqlPool,
    user_id: i64,
    ore_id: i64,
    amount: i32,
    max_allowed: i32,
) {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO UserOreAsset (userId, oreId, amount, maxAllowed) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(ore_id)
        .bind(amount)
        .bind(max_allowed),
    )
    .await;
}

pub async fn insert_committed_pending_changes(pool: &MySqlPool, robot_id: i64) {
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
}

pub async fn insert_mining_queue(pool: &MySqlPool, mining_area_id: i64, robot_id: i64) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningQueue (miningAreaId, robotId, miningEndTime) \
             VALUES (?, ?, NULL)",
        )
        .bind(mining_area_id)
        .bind(robot_id),
    )
    .await
}

pub async fn insert_claimed_mining_queue(
    pool: &MySqlPool,
    mining_area_id: i64,
    robot_id: i64,
    rally_result_id: i64,
) -> i64 {
    insert_row_id(
        pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, miningEndTime, claimed) \
             VALUES (?, ?, ?, TIMESTAMPADD(SECOND, -60, NOW()), true)",
        )
        .bind(mining_area_id)
        .bind(robot_id)
        .bind(rally_result_id),
    )
    .await
}

pub async fn cleanup_created_user(pool: &MySqlPool, user_id: i64) {
    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserRobotPartAsset WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserOreAsset WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserMiningArea WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserAchievement WHERE userId = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;
}

pub async fn cleanup_claimed_queue_fixture(
    pool: &MySqlPool,
    user_id: i64,
    robot_id: i64,
    mining_area_id: i64,
    ore_id: i64,
    ore_price_id: i64,
    remaining_queue_ids: &[i64],
) {
    for queue_id in remaining_queue_ids {
        let rally_result_id: Option<i64> =
            sqlx::query_scalar("SELECT rallyResultId FROM MiningQueue WHERE id = ?")
                .bind(queue_id)
                .fetch_optional(pool)
                .await
                .expect("failed to load rally result id");
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(pool)
            .await;
        if let Some(rally_result_id) = rally_result_id {
            let still_referenced: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM MiningQueue WHERE rallyResultId = ? LIMIT 1",
            )
            .bind(rally_result_id)
            .fetch_optional(pool)
            .await
            .expect("failed to check rally references");
            if still_referenced.is_none() {
                let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
                    .bind(rally_result_id)
                    .execute(pool)
                    .await;
            }
        }
    }

    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(robot_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(pool)
        .await;
    cleanup_created_user(pool, user_id).await;
}
