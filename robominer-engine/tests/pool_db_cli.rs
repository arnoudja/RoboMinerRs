#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn pool_rally_persist_updates_pool_item_tables() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;
    let record = robominer_db::CompletedPoolRallyRecord {
        items: vec![robominer_db::CompletedPoolItemRecord {
            pool_item_id: fixture.pool_item_id,
            score: 7.25,
            ore_results: vec![robominer_db::CompletedPoolItemOreRecord {
                ore_id: fixture.ore_id,
                amount: 4,
            }],
        }],
    };

    robominer_db::persist_completed_pool_rally(&pool, &record)
        .await
        .expect("failed to persist pool rally");

    fixture.assert_persisted(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_pool_persist_updates_pool_item_tables() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "pool".to_string(),
        "--pool-id".to_string(),
        fixture.pool_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally pool --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Pool rally complete") && stdout.contains("Persisted pool rally"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_cli_persisted(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_pool_until_complete_persists_until_required_runs() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "pool".to_string(),
        "--pool-id".to_string(),
        fixture.pool_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
        "--until-complete".to_string(),
        "--max-rallies".to_string(),
        "5".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally pool --until-complete --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Pool rally complete")
            && stdout.contains("Persisted pool rally")
            && stdout.contains("Pool repeat complete: ran=1"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    fixture.assert_cli_persisted(&pool).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn run_pool_dry_run_does_not_persist() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;

    let before = sqlx::query("SELECT totalScore, runsDone FROM PoolItem WHERE id = ?")
        .bind(fixture.pool_item_id)
        .fetch_one(&pool)
        .await
        .expect("load pool item before dry run");
    let before_score: f64 = before.try_get("totalScore").unwrap();
    let before_runs: i32 = before.try_get("runsDone").unwrap();

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally".to_string(),
        "pool".to_string(),
        "--pool-id".to_string(),
        fixture.pool_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally pool dry run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Dry run: no database writes performed"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let after = sqlx::query("SELECT totalScore, runsDone FROM PoolItem WHERE id = ?")
        .bind(fixture.pool_item_id)
        .fetch_one(&pool)
        .await
        .expect("load pool item after dry run");
    let after_score: f64 = after.try_get("totalScore").unwrap();
    let after_runs: i32 = after.try_get("runsDone").unwrap();
    assert_eq!(after_score, before_score);
    assert_eq!(after_runs, before_runs);

    fixture.cleanup(&pool).await;
}
