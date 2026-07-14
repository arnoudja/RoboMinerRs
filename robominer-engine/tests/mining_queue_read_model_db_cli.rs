mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn mining_queue_states_report_active_and_queued_items() {
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
        "mining-queue-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected mining-queue-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let active_state = find_queue_state_line(&stdout, fixture.active_queue_id);
    let queued_state = find_queue_state_line(&stdout, fixture.queued_queue_id);

    assert_eq!(active_state[1], fixture.robot_id.to_string());
    assert_eq!(active_state[2], "MINING");
    assert_eq!(queued_state[1], fixture.robot_id.to_string());
    assert_eq!(queued_state[2], "QUEUED");

    let active_time_left: i64 = active_state[3].parse().expect("active time should parse");
    let queued_time_left: i64 = queued_state[3].parse().expect("queued time should parse");
    assert!(active_time_left > 0);
    assert!(queued_time_left > active_time_left);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn mining_queue_page_states_report_page_read_model() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;

    sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
        .bind(fixture.user_id)
        .bind(fixture.mining_area_id)
        .execute(&pool)
        .await
        .expect("failed to grant mining area");
    sqlx::query("INSERT INTO OrePriceAmount (orePriceId, oreId, amount) VALUES (?, ?, 4)")
        .bind(fixture.ore_price_id)
        .bind(fixture.ore_id)
        .execute(&pool)
        .await
        .expect("failed to insert ore price amount");
    sqlx::query(
        "INSERT INTO MiningAreaOreSupply (miningAreaId, oreId, supply, radius) \
         VALUES (?, ?, 7, 3)",
    )
    .bind(fixture.mining_area_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert mining area ore supply");
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
    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 2, 42.5)",
    )
    .bind(fixture.robot_id)
    .bind(fixture.mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot mining area score");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "mining-queue-page-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected mining-queue-page-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let robot_line = find_prefixed_line(&stdout, &format!("R\t{}\t", fixture.robot_id));
    assert!(robot_line[2].ends_with("-robot"));

    let area_line = find_prefixed_line(&stdout, &format!("A\t{}\t", fixture.mining_area_id));
    assert!(area_line[2].ends_with("-area"));
    assert_eq!(area_line[3], "0");
    assert_eq!(area_line[4], "10");
    assert_eq!(area_line[5], "1");
    assert_eq!(area_line[6], "4");
    assert_eq!(area_line[7], "4");

    let cost_line = find_prefixed_line(
        &stdout,
        &format!("C\t{}\t{}\t", fixture.mining_area_id, fixture.ore_id),
    );
    assert!(cost_line[3].ends_with("-ore"));
    assert_eq!(cost_line[4], "4");

    let supply_line = find_prefixed_line(
        &stdout,
        &format!("S\t{}\t{}\t", fixture.mining_area_id, fixture.ore_id),
    );
    assert!(supply_line[3].ends_with("-ore"));
    assert_eq!(supply_line[4], "7");
    assert_eq!(supply_line[5], "3");

    let yield_line = find_prefixed_line(
        &stdout,
        &format!("H\t{}\t{}\t", fixture.mining_area_id, fixture.ore_id),
    );
    assert!(yield_line[3].ends_with("-ore"));
    assert_eq!(yield_line[4], "25");

    let active_queue_line =
        find_prefixed_line(&stdout, &format!("Q\t{}\t", fixture.active_queue_id));
    assert_eq!(active_queue_line[2], fixture.robot_id.to_string());
    assert_eq!(active_queue_line[3], fixture.mining_area_id.to_string());
    assert!(active_queue_line[4].ends_with("-area"));
    assert_eq!(active_queue_line[5], "MINING");
    active_queue_line[6]
        .parse::<i64>()
        .expect("active time should parse");

    let queued_queue_line =
        find_prefixed_line(&stdout, &format!("Q\t{}\t", fixture.queued_queue_id));
    assert_eq!(queued_queue_line[2], fixture.robot_id.to_string());
    assert_eq!(queued_queue_line[5], "QUEUED");

    let score_line = find_prefixed_line(
        &stdout,
        &format!("P\t{}\t{}\t", fixture.robot_id, fixture.mining_area_id),
    );
    assert_eq!(score_line[3], "42.5");

    fixture.cleanup(&pool).await;
}
