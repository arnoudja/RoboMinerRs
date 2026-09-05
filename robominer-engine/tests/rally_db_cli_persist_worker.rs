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
