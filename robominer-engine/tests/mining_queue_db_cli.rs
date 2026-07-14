mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn enqueue_mining_fill_deducts_costs_and_inserts_until_queue_limit() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestEnqueueMiningFixture::create(&pool, 3, 25, 4, true).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "enqueue-mining".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-id".to_string(),
        fixture.robot_id.to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
        "--fill".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected enqueue-mining --fill to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Enqueued 3 mining run(s)"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_queue_and_asset(&pool, 3, 13).await;

    let login_updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM User \
         WHERE id = ? AND lastLoginTime > TIMESTAMPADD(MINUTE, -1, NOW())",
    )
    .bind(fixture.user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to check last login time");
    assert_eq!(login_updated, 1, "enqueue-mining should refresh last login time");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn enqueue_mining_insufficient_funds_rolls_back() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestEnqueueMiningFixture::create(&pool, 2, 3, 4, true).await;

    sqlx::query("UPDATE User SET lastLoginTime = TIMESTAMPADD(DAY, -1, NOW()) WHERE id = ?")
        .bind(fixture.user_id)
        .execute(&pool)
        .await
        .expect("failed to backdate login time");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "enqueue-mining".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-id".to_string(),
        fixture.robot_id.to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        !output.status.success(),
        "expected enqueue-mining to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.contains("insufficient funds"),
        "unexpected stderr:\n{stderr}"
    );

    fixture.assert_queue_and_asset(&pool, 0, 3).await;

    let login_updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM User \
         WHERE id = ? AND lastLoginTime > TIMESTAMPADD(MINUTE, -1, NOW())",
    )
    .bind(fixture.user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to check last login time");
    assert_eq!(
        login_updated, 0,
        "failed enqueue-mining should not refresh last login time"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn enqueue_mining_rejects_unavailable_area_and_wrong_owner() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestEnqueueMiningFixture::create(&pool, 2, 20, 4, false).await;

    let unavailable_output = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "enqueue-mining".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--robot-id".to_string(),
        fixture.robot_id.to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
    ]);
    let (_, unavailable_stderr) = output_text(&unavailable_output);
    assert!(
        !unavailable_output.status.success(),
        "expected unavailable area rejection"
    );
    assert!(
        unavailable_stderr.contains("mining area is not available"),
        "unexpected stderr:\n{unavailable_stderr}"
    );

    let wrong_owner_output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "enqueue-mining".to_string(),
        "--user-id".to_string(),
        fixture.other_user_id.to_string(),
        "--robot-id".to_string(),
        fixture.robot_id.to_string(),
        "--mining-area-id".to_string(),
        fixture.mining_area_id.to_string(),
    ]);
    let (_, wrong_owner_stderr) = output_text(&wrong_owner_output);
    assert!(
        !wrong_owner_output.status.success(),
        "expected wrong owner rejection"
    );
    assert!(
        wrong_owner_stderr.contains("unknown robot"),
        "unexpected stderr:\n{wrong_owner_stderr}"
    );

    fixture.assert_queue_and_asset(&pool, 0, 20).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_deletes_queued_item() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "cancel-mining-queue".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--mining-queue-id".to_string(),
        fixture.queued_queue_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected cancel-mining-queue to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Canceled mining queue"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture
        .assert_queue_exists(&pool, fixture.active_queue_id, true)
        .await;
    fixture
        .assert_queue_exists(&pool, fixture.queued_queue_id, false)
        .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_rejects_wrong_owner_and_unknown_queue() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;

    let wrong_owner = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "cancel-mining-queue".to_string(),
        "--user-id".to_string(),
        fixture.other_user_id.to_string(),
        "--mining-queue-id".to_string(),
        fixture.queued_queue_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&wrong_owner);
    assert!(
        !wrong_owner.status.success(),
        "expected wrong owner cancel to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("belongs to another user"),
        "unexpected stderr:\n{stderr}"
    );

    let unknown_queue = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "cancel-mining-queue".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--mining-queue-id".to_string(),
        (fixture.queued_queue_id + 1_000_000).to_string(),
    ]);
    let (stdout, stderr) = output_text(&unknown_queue);
    assert!(
        !unknown_queue.status.success(),
        "expected unknown queue cancel to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("unknown mining queue item"),
        "unexpected stderr:\n{stderr}"
    );

    fixture
        .assert_queue_exists(&pool, fixture.queued_queue_id, true)
        .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_rejects_active_and_rally_backed_items() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;
    let rally_backed_queue_id = fixture.add_rally_backed_queue(&pool).await;

    let active_queue = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "cancel-mining-queue".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--mining-queue-id".to_string(),
        fixture.active_queue_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&active_queue);
    assert!(
        !active_queue.status.success(),
        "expected active queue cancel to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("not cancelable"),
        "unexpected stderr:\n{stderr}"
    );

    let rally_backed = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "cancel-mining-queue".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--mining-queue-id".to_string(),
        rally_backed_queue_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&rally_backed);
    assert!(
        !rally_backed.status.success(),
        "expected rally-backed queue cancel to fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("not cancelable"),
        "unexpected stderr:\n{stderr}"
    );

    fixture
        .assert_queue_exists(&pool, fixture.active_queue_id, true)
        .await;
    fixture
        .assert_queue_exists(&pool, rally_backed_queue_id, true)
        .await;
    fixture.cleanup(&pool).await;
}

