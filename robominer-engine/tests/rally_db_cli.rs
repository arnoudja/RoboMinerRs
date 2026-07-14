mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn run_rally_persist_writes_completed_rally_tables() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "run-rally".to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected run-rally --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Rally complete") && stdout.contains("Persisted rally result"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_persisted(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_rallies_once_persist_advances_ready_queue() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;
    let area_name: String = sqlx::query_scalar("SELECT areaName FROM MiningArea WHERE id = ?")
        .bind(fixture.mining_area_id)
        .fetch_one(&pool)
        .await
        .expect("failed to load mining area name");

    // Let miningTime/recharge windows elapse so the queue is rally-ready before polling.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "run-rallies".to_string(),
        "--once".to_string(),
        "--persist".to_string(),
        "--seed".to_string(),
        "0".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected run-rallies --once --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Processed mining areas:") && stdout.contains("ran="),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "Processing mining area {} ({area_name})",
            fixture.mining_area_id
        )),
        "run-rallies should visit the fixture mining area\nstdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let queue = sqlx::query(
        "SELECT rallyResultId, miningEndTime IS NOT NULL AS ended \
         FROM MiningQueue \
         WHERE id = ?",
    )
    .bind(fixture.mining_queue_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load queue row after run-rallies");

    let rally_result_id: Option<i64> = queue.try_get("rallyResultId").unwrap();
    let ended: i8 = queue.try_get("ended").unwrap();
    assert!(
        rally_result_id.is_some(),
        "fixture queue should reference a persisted rally result"
    );
    assert_eq!(ended, 1, "fixture queue should be marked finished");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cleanup_old_claimed_mining_queue_items_keeps_recent_history() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-cleanup-history");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();").await;
    let ore_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
    )
    .await;
    let ore_price_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
            .bind(format!("{prefix}-price")),
    )
    .await;
    let mining_area_id = insert_row_id(
        &pool,
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

    let mut queue_ids = Vec::new();
    for _ in 0..14 {
        let rally_result_id = insert_row_id(
            &pool,
            sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('{}')"),
        )
        .await;
        queue_ids.push(
            insert_claimed_mining_queue(&pool, mining_area_id, robot_id, rally_result_id).await,
        );
    }

    let summary =
        robominer_db::cleanup_old_claimed_mining_queue_items_for_robot(&pool, robot_id)
            .await
            .expect("cleanup should succeed");

    assert_eq!(summary.queues_deleted, 2);
    assert_eq!(summary.rally_results_deleted, 2);

    let remaining_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM MiningQueue WHERE robotId = ? AND claimed = true ORDER BY id",
    )
    .bind(robot_id)
    .fetch_all(&pool)
    .await
    .expect("failed to load remaining claimed queue rows");

    assert_eq!(remaining_ids.len(), 12);
    assert_eq!(remaining_ids, queue_ids[2..]);

    for deleted_queue_id in &queue_ids[..2] {
        let ore_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM MiningOreResult WHERE miningQueueId = ?",
        )
        .bind(deleted_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count ore rows");
        assert_eq!(ore_rows, 0);
    }

    cleanup_claimed_queue_fixture(
        &pool,
        user_id,
        robot_id,
        mining_area_id,
        ore_id,
        ore_price_id,
        &queue_ids[2..],
    )
    .await;
}

#[tokio::test]
#[serial]
async fn cleanup_old_claimed_mining_queue_items_keeps_shared_rally_results() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-cleanup-shared");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;
    let robot_a_id = insert_robot(&pool, user_id, &format!("{prefix}-robot-a"), "mine();").await;
    let robot_b_id = insert_robot(&pool, user_id, &format!("{prefix}-robot-b"), "mine();").await;
    let ore_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO Ore (oreName) VALUES (?)").bind(format!("{prefix}-ore")),
    )
    .await;
    let ore_price_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
            .bind(format!("{prefix}-price")),
    )
    .await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(robot_a_id),
    )
    .await;
    let shared_rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('{}')"),
    )
    .await;
    let shared_queue_id =
        insert_claimed_mining_queue(&pool, mining_area_id, robot_b_id, shared_rally_result_id)
            .await;

    let mut queue_ids = vec![insert_claimed_mining_queue(
        &pool,
        mining_area_id,
        robot_a_id,
        shared_rally_result_id,
    )
    .await];
    for _ in 0..13 {
        let rally_result_id = insert_row_id(
            &pool,
            sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('{}')"),
        )
        .await;
        queue_ids.push(
            insert_claimed_mining_queue(&pool, mining_area_id, robot_a_id, rally_result_id).await,
        );
    }

    let summary =
        robominer_db::cleanup_old_claimed_mining_queue_items_for_robot(&pool, robot_a_id)
            .await
            .expect("cleanup should succeed");

    assert_eq!(summary.queues_deleted, 2);
    assert_eq!(summary.rally_results_deleted, 1);

    let shared_rally_exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM RallyResult WHERE id = ?")
            .bind(shared_rally_result_id)
            .fetch_optional(&pool)
            .await
            .expect("failed to load shared rally result");
    assert_eq!(shared_rally_exists, Some(shared_rally_result_id));

    let shared_queue_exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM MiningQueue WHERE id = ?")
            .bind(shared_queue_id)
            .fetch_optional(&pool)
            .await
            .expect("failed to load shared queue row");
    assert_eq!(shared_queue_exists, Some(shared_queue_id));

    cleanup_claimed_queue_fixture(
        &pool,
        user_id,
        robot_a_id,
        mining_area_id,
        ore_id,
        ore_price_id,
        &queue_ids[2..],
    )
    .await;

    let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
        .bind(shared_queue_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(robot_b_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(shared_rally_result_id)
        .execute(&pool)
        .await;
}

