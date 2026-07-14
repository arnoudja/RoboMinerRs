mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn pool_rally_persist_updates_pool_item_tables() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
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
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "run-pool".to_string(),
        "--pool-id".to_string(),
        fixture.pool_id.to_string(),
        "--seed".to_string(),
        "0".to_string(),
        "--persist".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected run-pool --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
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
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestPoolFixture::create(&pool).await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "run-pool".to_string(),
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
        "expected run-pool --until-complete --persist to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
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
