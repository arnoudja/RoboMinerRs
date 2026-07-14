mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn mining_area_overview_states_report_lifetime_percentages() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;

    sqlx::query(
        "INSERT INTO MiningAreaLifetimeResult \
         (miningAreaId, oreId, totalAmount, totalContainerSize) \
         VALUES (?, ?, 25, 100)",
    )
    .bind(fixture.mining_area_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert mining area lifetime result");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "mining-area-overview-states".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected mining-area-overview-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let ore_prefix = format!("O\t{}\t", fixture.ore_id);
    let ore_line = stdout
        .lines()
        .find(|line| line.starts_with(&ore_prefix))
        .unwrap_or_else(|| panic!("expected overview ore row in stdout:\n{stdout}"));
    assert!(ore_line.ends_with("-ore"));

    let area_prefix = format!("A\t{}\t", fixture.mining_area_id);
    let area_line = stdout
        .lines()
        .find(|line| line.starts_with(&area_prefix))
        .unwrap_or_else(|| panic!("expected overview area row in stdout:\n{stdout}"));
    let area_fields: Vec<&str> = area_line.split('\t').collect();
    assert!(area_fields[2].ends_with("-area"));
    assert_eq!(area_fields[3], "25");

    assert!(
        stdout.contains(&format!(
            "P\t{}\t{}\t25",
            fixture.mining_area_id, fixture.ore_id
        )),
        "expected overview percentage row in stdout:\n{stdout}"
    );

    let _ = sqlx::query("DELETE FROM MiningAreaLifetimeResult WHERE miningAreaId = ?")
        .bind(fixture.mining_area_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}

