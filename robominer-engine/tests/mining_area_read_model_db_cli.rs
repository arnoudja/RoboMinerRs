mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn mining_area_scores_report_user_robot_scores() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;

    sqlx::query(
        "INSERT INTO RobotMiningAreaScore (robotId, miningAreaId, totalRuns, score) \
         VALUES (?, ?, 3, 42.5)",
    )
    .bind(fixture.robot_id)
    .bind(fixture.mining_area_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot mining area score");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "mining-area-scores".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected mining-area-scores to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let score_state = find_score_state_line(&stdout, fixture.robot_id, fixture.mining_area_id);
    assert_eq!(score_state[2], "42.5");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn mining_result_states_report_result_details() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = TestCancelMiningQueueFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('test-result-state')"),
    )
    .await;
    let mining_queue_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, score, miningEndTime, claimed) \
             VALUES (?, ?, ?, 0, 123.4, TIMESTAMPADD(SECOND, -10, NOW()), TRUE)",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.robot_id)
        .bind(rally_result_id),
    )
    .await;

    sqlx::query(
        "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, tax) \
         VALUES (?, ?, 20, 3)",
    )
    .bind(mining_queue_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("failed to insert mining ore result");
    sqlx::query(
        "INSERT INTO RobotActionsDone (miningQueueId, actionType, amount) \
         VALUES (?, 6, 5)",
    )
    .bind(mining_queue_id)
    .execute(&pool)
    .await
    .expect("failed to insert robot actions done");

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "mining-result-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--max-results".to_string(),
        "10".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected mining-result-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let robot_fields = find_prefixed_line(&stdout, &format!("R\t{}\t", fixture.robot_id));
    assert_eq!(robot_fields.len(), 3);
    assert!(robot_fields[2].ends_with("-robot"));

    let result_prefix = format!("Q\t{}\t{mining_queue_id}\t", fixture.robot_id);
    let result_line = stdout
        .lines()
        .find(|line| line.starts_with(&result_prefix))
        .unwrap_or_else(|| panic!("expected mining result row in stdout:\n{stdout}"));
    let result_fields: Vec<&str> = result_line.split('\t').collect();
    assert!(
        result_fields[3].ends_with("-area"),
        "unexpected mining area name: {}",
        result_fields[3]
    );
    assert_eq!(result_fields[4], rally_result_id.to_string());
    assert_eq!(result_fields[5], "123.4");
    assert_eq!(result_fields[6], "20");
    assert_eq!(result_fields[7], "3");
    assert_eq!(result_fields[8], "17");
    result_fields[9]
        .parse::<i64>()
        .expect("creation time millis should parse");
    result_fields[10]
        .parse::<i64>()
        .expect("mining end time millis should parse");

    let ore_prefix = format!("O\t{mining_queue_id}\t{}\t", fixture.ore_id);
    let ore_line = stdout
        .lines()
        .find(|line| line.starts_with(&ore_prefix))
        .unwrap_or_else(|| panic!("expected mining ore result row in stdout:\n{stdout}"));
    let ore_fields: Vec<&str> = ore_line.split('\t').collect();
    assert!(ore_fields[3].ends_with("-ore"));
    assert_eq!(ore_fields[4], "20");
    assert_eq!(ore_fields[5], "3");
    assert_eq!(ore_fields[6], "17");
    assert!(
        stdout.contains(&format!("D\t{mining_queue_id}\t6\t5")),
        "expected robot action result row in stdout:\n{stdout}"
    );

    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}

