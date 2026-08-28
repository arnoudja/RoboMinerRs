#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn run_rally_persist_writes_completed_rally_tables() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "run".to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally run --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
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
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;
    fixture.make_rally_ready(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "rallies".to_string(),
        "--once".to_string(),
        "--persist".to_string(),
        "--seed".to_string(),
        "0".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally rallies --once --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Processed mining areas:") && stdout.contains("ran="),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Rally complete") && stdout.contains("Persisted rally result"),
        "rally rallies should run and persist the fixture queue\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("Processing mining area"),
        "poll loop should not log per-area processing noise\nstdout:\n{stdout}"
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
    .expect("failed to load queue row after rally rallies");

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
async fn run_rallies_once_persist_claims_ready_wallet_results() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestClaimResultsFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "rallies".to_string(),
        "--once".to_string(),
        "--persist".to_string(),
        "--seed".to_string(),
        "0".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally rallies --once --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        stdout.matches("Wallet claim pass:").count(),
        1,
        "persist cycle should run exactly one wallet claim pass\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Claimed 1 mining result(s)") && stdout.contains("Added to wallet:"),
        "wallet claim pass should credit the fixture user\nstdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_claimed(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cleanup_old_claimed_mining_queue_items_keeps_recent_history() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
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
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(ai_robot_id),
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

    let summary = robominer_db::cleanup_old_claimed_mining_queue_items_for_robot(&pool, robot_id)
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
        let ore_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM MiningOreResult WHERE miningQueueId = ?")
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
    let Some(database_url) = robominer_test_support::require_test_db() else {
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
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 1, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(ai_robot_id),
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

    let mut queue_ids = vec![
        insert_claimed_mining_queue(&pool, mining_area_id, robot_a_id, shared_rally_result_id)
            .await,
    ];
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

    let summary = robominer_db::cleanup_old_claimed_mining_queue_items_for_robot(&pool, robot_a_id)
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

#[tokio::test]
#[serial]
async fn run_rally_dry_run_does_not_persist() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "run".to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally dry run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Dry run: no database writes performed"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let rally_result_id: Option<i64> =
        sqlx::query_scalar("SELECT rallyResultId FROM MiningQueue WHERE id = ?")
            .bind(fixture.mining_queue_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load queue after dry run");
    assert!(
        rally_result_id.is_none(),
        "dry run must not attach a rally result to the queue"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_rally_result_data_file_overrides_sim_payload() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    let payload = r#"{"v":2,"override":true,"robots":{"robot":[]},"ground":{},"oreTypes":{}}"#;
    let path = std::env::temp_dir().join(format!(
        "robominer-rally-result-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, payload).expect("write result data file");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "rally".to_string(),
        "run".to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
        "--result-data-file".to_string(),
        path.display().to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);
    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "expected rally with result-data-file to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let rally_result_id: Option<i64> =
        sqlx::query_scalar("SELECT rallyResultId FROM MiningQueue WHERE id = ?")
            .bind(fixture.mining_queue_id)
            .fetch_one(&pool)
            .await
            .expect("queue should reference rally result");
    let rally_result_id = rally_result_id.expect("rallyResultId");
    let stored: String = sqlx::query_scalar("SELECT resultData FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .fetch_one(&pool)
        .await
        .expect("load stored result data");
    assert_eq!(stored, payload);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_rally_missing_result_data_file_fails() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "run".to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
        "--result-data-file".to_string(),
        "/tmp/robominer-missing-result-data-file.json".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "missing result-data-file should fail"
    );
    assert!(
        stderr.contains("failed to read result data file")
            || stdout.contains("failed to read result data file"),
        "expected read error:\nstdout: {stdout}\nstderr: {stderr}"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_next_mining_rally_leases_at_most_rally_size() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_test_prefix("rust-claim-truncate");
    let user_id = insert_test_user(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;
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
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let mining_area_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 60, 0, ?)",
        )
        .bind(format!("{prefix}-area"))
        .bind(ore_price_id)
        .bind(ai_robot_id),
    )
    .await;
    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
             VALUES (?, ?, 10, 2)",
        )
        .bind(mining_area_id)
        .bind(ore_id),
    )
    .await;

    let mut queue_ids = Vec::new();
    let mut robot_ids = Vec::new();
    let mut user_ids = Vec::new();
    for index in 0..6 {
        let owner_id = insert_test_user(
            &pool,
            &format!("{prefix}-user-{index}"),
            &format!("{prefix}-{index}@example.invalid"),
            "test-password",
        )
        .await;
        user_ids.push(owner_id);
        let robot_id = insert_robot(
            &pool,
            owner_id,
            &format!("{prefix}-robot-{index}"),
            "mine();",
        )
        .await;
        robot_ids.push(robot_id);
        let queue_id = insert_row_id(
            &pool,
            sqlx::query(
                "INSERT INTO MiningQueue (miningAreaId, robotId, creationTime, miningEndTime) \
                 VALUES (?, ?, TIMESTAMPADD(SECOND, -3600, NOW()), NULL)",
            )
            .bind(mining_area_id)
            .bind(robot_id),
        )
        .await;
        queue_ids.push(queue_id);
    }

    let claimed =
        robominer_db::claim_next_mining_rally_queue_for_area(&pool, mining_area_id, 4, 10)
            .await
            .expect("claim should succeed")
            .expect("rally should be ready");
    assert_eq!(claimed.len(), 4, "claim must lease at most rally_size rows");

    let leased: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue \
         WHERE miningAreaId = ? \
           AND processingLeaseUntil IS NOT NULL \
           AND processingLeaseUntil > NOW()",
    )
    .bind(mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("count leased rows");
    assert_eq!(leased, 4);

    let remaining = robominer_db::list_next_mining_rally_queue_for_area(&pool, mining_area_id)
        .await
        .expect("list remaining free queue heads");
    assert_eq!(
        remaining.len(),
        2,
        "unclaimed free heads should remain claimable"
    );

    for queue_id in queue_ids {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM MiningAreaOreSupply WHERE miningAreaId = ?")
        .bind(mining_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(mining_area_id)
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
    let _ = sqlx::query("DELETE FROM Ore WHERE id = ?")
        .bind(ore_id)
        .execute(&pool)
        .await;
    for robot_id in robot_ids {
        let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
            .bind(robot_id)
            .execute(&pool)
            .await;
    }
    for owner_id in user_ids {
        cleanup_created_user(&pool, owner_id).await;
    }
    cleanup_created_user(&pool, user_id).await;
}

#[tokio::test]
#[serial]
async fn list_next_claim_rally_candidates_reports_busy_seconds() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRallyFixture::create(&pool).await;

    sqlx::query(
        "UPDATE Robot SET miningEndTime = TIMESTAMPADD(SECOND, 25, NOW()), \
             rechargeEndTime = TIMESTAMPADD(SECOND, 40, NOW()) \
         WHERE id = ?",
    )
    .bind(fixture.queued_robot_id)
    .execute(&pool)
    .await
    .expect("set robot busy");

    let candidates = robominer_db::list_next_claim_rally_candidates(&pool)
        .await
        .expect("list candidates");
    let match_candidate = candidates
        .iter()
        .find(|c| c.mining_area_id == fixture.mining_area_id)
        .expect("fixture area should appear in candidates");
    assert!(
        match_candidate.busy_seconds >= 25,
        "busy_seconds should reflect mining/recharge end: {match_candidate:?}"
    );

    let area_candidates: Vec<_> = candidates
        .iter()
        .filter(|c| c.mining_area_id == fixture.mining_area_id)
        .cloned()
        .collect();
    let delay = robominer_domain::next_claimable_rally_delay_seconds(&area_candidates);
    assert!(
        delay.is_some_and(|d| d >= 25),
        "partial busy queue should wait for free/expiry, got {delay:?}"
    );

    fixture.cleanup(&pool).await;
}
