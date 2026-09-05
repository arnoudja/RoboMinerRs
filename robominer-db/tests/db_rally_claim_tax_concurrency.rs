#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_test_support::{
    ClaimResultsFixture, insert_ai_robot, insert_area_supply, insert_finished_queue,
    insert_mining_area, insert_ore, insert_ore_price, insert_ore_result_with_depot, insert_robot,
    insert_row_id, insert_user, insert_user_ore_asset, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn claim_taxes_container_and_depot_ore_separately() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("split-tax");
    let ore_id = insert_ore(&pool, &format!("{prefix}-ore")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 20).await;
    insert_area_supply(&pool, mining_area_id, ore_id, 10, 2).await;
    let queue_id = insert_finished_queue(&pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result_with_depot(&pool, queue_id, ore_id, 100, 40).await;
    insert_user_ore_asset(&pool, user_id, ore_id, 0, 1000).await;

    let claimed = robominer_db::claim_user_results(&pool, user_id)
        .await
        .expect("claim should succeed");
    assert_eq!(claimed.claimed_queues, 1);
    assert_eq!(claimed.ore_rewards.len(), 1);
    assert_eq!(claimed.ore_rewards[0].ore_id, ore_id);
    assert_eq!(claimed.ore_rewards[0].reward, 84);

    let tax: i32 =
        sqlx::query_scalar("SELECT tax FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?")
            .bind(queue_id)
            .bind(ore_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load tax");
    assert_eq!(tax, 16);

    let wallet: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(user_id)
            .bind(ore_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load wallet");
    assert_eq!(wallet, 84);
}

#[tokio::test]
#[serial]
async fn next_wallet_claim_delay_seconds_uses_soonest_unclaimed_end_time() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("claim-delay");
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let user_id = insert_user(&pool, &prefix).await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 20).await;

    // Shared local DBs may already have finished unclaimed runs from play/other tests.
    // Those shorten delay to 1s, so only assert the empty/future paths when the DB is clear.
    let preexisting_ready: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue \
         WHERE miningEndTime IS NOT NULL AND miningEndTime <= NOW() AND claimed = false",
    )
    .fetch_one(&pool)
    .await
    .expect("preexisting ready count");

    if preexisting_ready == 0 {
        let empty_delay = robominer_db::next_wallet_claim_delay_seconds(&pool, 45)
            .await
            .expect("empty claim delay");
        assert_eq!(
            empty_delay, 45,
            "with no unclaimed future runs, delay should fall back to max"
        );
    }

    let queue_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue (miningAreaId, robotId, miningEndTime, claimed) \
             VALUES (?, ?, TIMESTAMPADD(SECOND, 30, NOW()), false)",
        )
        .bind(mining_area_id)
        .bind(robot_id),
    )
    .await;

    if preexisting_ready == 0 {
        let delay = robominer_db::next_wallet_claim_delay_seconds(&pool, 60)
            .await
            .expect("claim delay with future queue");
        assert!(
            (25..=35).contains(&delay),
            "expected ~30s until finish, got {delay}"
        );

        let capped = robominer_db::next_wallet_claim_delay_seconds(&pool, 10)
            .await
            .expect("capped claim delay");
        assert_eq!(capped, 10, "delay must respect max_sleep_seconds");
    }

    sqlx::query(
        "UPDATE MiningQueue SET miningEndTime = TIMESTAMPADD(SECOND, -5, NOW()) WHERE id = ?",
    )
    .bind(queue_id)
    .execute(&pool)
    .await
    .expect("failed to mark queue ready-now");

    let ready_delay = robominer_db::next_wallet_claim_delay_seconds(&pool, 60)
        .await
        .expect("ready-now claim delay");
    assert_eq!(
        ready_delay, 1,
        "finished unclaimed runs must shorten poll sleep to 1s, got {ready_delay}"
    );

    let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
        .bind(queue_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM AIRobot WHERE id = ?")
        .bind(ai_robot_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(ore_price_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn claimable_mining_queue_query_can_use_claimable_index() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");

    robominer_db::run_embedded_migrations(&pool)
        .await
        .expect("apply embedded migrations including claimable index");

    let index_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM information_schema.statistics \
         WHERE table_schema = DATABASE() \
           AND table_name = 'MiningQueue' \
           AND index_name = 'idx_mining_queue_claimable'",
    )
    .fetch_one(&pool)
    .await
    .expect("lookup claimable index");
    assert_eq!(
        index_columns, 2,
        "expected (claimed, miningEndTime) claimable index columns"
    );

    // EXPLAIN FORMAT=JSON keeps the plan in one string column (sqlx-friendly).
    let plan: String = sqlx::query_scalar(
        "EXPLAIN FORMAT=JSON \
         SELECT DISTINCT Robot.userId \
         FROM MiningQueue \
         INNER JOIN Robot ON Robot.id = MiningQueue.robotId \
         WHERE MiningQueue.miningEndTime IS NOT NULL \
           AND MiningQueue.miningEndTime <= NOW() \
           AND MiningQueue.claimed = false \
         ORDER BY Robot.userId",
    )
    .fetch_one(&pool)
    .await
    .expect("explain claimable scan");
    assert!(
        plan.contains("idx_mining_queue_claimable"),
        "expected EXPLAIN JSON to mention idx_mining_queue_claimable, got:\n{plan}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[serial]
async fn claim_user_results_concurrent_workers_do_not_double_credit() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ClaimResultsFixture::create(&pool).await;

    let mut handles = Vec::with_capacity(4);
    for _ in 0..4 {
        let pool = pool.clone();
        let user_id = fixture.user_id;
        handles.push(tokio::spawn(async move {
            robominer_db::claim_user_results(&pool, user_id)
                .await
                .expect("claim should not fail at sql layer")
        }));
    }

    let mut total_claimed_queues = 0_u64;
    for handle in handles {
        let result = handle.await.expect("claim task should complete");
        total_claimed_queues += result.claimed_queues;
    }

    assert_eq!(
        total_claimed_queues, 1,
        "exactly one worker should claim the finished queue"
    );

    let claimed: i8 = sqlx::query_scalar("SELECT claimed FROM MiningQueue WHERE id = ?")
        .bind(fixture.mining_queue_id)
        .fetch_one(&pool)
        .await
        .expect("claimed flag");
    assert_eq!(claimed, 1);

    let wallet: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(fixture.user_id)
            .bind(fixture.primary_ore_id)
            .fetch_one(&pool)
            .await
            .expect("wallet amount");
    // Fixture starts at 2; reward after 25% tax on 10 is 8; cap maxAllowed=8 → wallet stays 8.
    assert_eq!(wallet, 8);

    fixture.cleanup(&pool).await;
}
