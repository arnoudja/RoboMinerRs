#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;
use serial_test::serial;

use support::*;

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
