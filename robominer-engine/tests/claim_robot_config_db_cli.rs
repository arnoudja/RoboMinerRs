mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn claim_results_updates_assets_totals_and_pending_changes() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestClaimResultsFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "claim-results".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-results to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Claimed 1 mining result(s)") && stdout.contains("Added to wallet:"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("+8"),
        "expected net ore reward in stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_claimed(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_results_includes_runs_whose_end_time_equals_now() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestClaimResultsFixture::create_with_mining_end_time(&pool, "NOW()").await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "claim-results".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected claim-results to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Claimed 1 mining result(s)"),
        "expected run ending now to be claimable\nstdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_claimed(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn list_robot_config_reconciles_stale_pending_changes_without_claim() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestClaimResultsFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "robot-config-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected robot-config-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let pending_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM PendingRobotChanges WHERE robotId = ?")
            .bind(fixture.robot_id)
            .fetch_one(&pool)
            .await
            .expect("failed to count pending robot changes");
    assert_eq!(
        pending_count, 0,
        "stale pending changes should be cleared when opening robot config"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn list_robot_config_applies_orphaned_pending_changes_when_idle() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestRobotConfigFixture::create(&pool, true, true, 8).await;

    let update_output = run_engine(&fixture.update_args(&database_url));
    let (update_stdout, update_stderr) = output_text(&update_output);
    assert!(
        update_output.status.success(),
        "expected update-robot-config to succeed\nstdout:\n{update_stdout}\nstderr:\n{update_stderr}"
    );

    sqlx::query(
        "UPDATE MiningQueue \
         SET miningEndTime = TIMESTAMPADD(SECOND, -5, NOW()) \
         WHERE robotId = ?",
    )
    .bind(fixture.robot_id)
    .execute(&pool)
    .await
    .expect("failed to mark mining queue finished");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "robot-config-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected robot-config-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_active_updated(&pool).await;
    fixture.assert_no_pending_changes(&pool).await;
    fixture.cleanup(&pool).await;
}

