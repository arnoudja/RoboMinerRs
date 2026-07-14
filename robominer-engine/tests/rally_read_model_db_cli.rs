mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn rally_view_state_reports_animation_legend_and_slots() {
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
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('var rallyViewTest = 1;')"),
    )
    .await;
    fixture.rally_result_id.set(Some(rally_result_id));

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime, claimed) \
             VALUES (?, ?, ?, 0, TIMESTAMPADD(SECOND, -10, NOW()), TRUE)",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.robot_id)
        .bind(rally_result_id),
    )
    .await;
    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime) \
             VALUES (?, ?, ?, 1, TIMESTAMPADD(SECOND, -10, NOW()))",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.ai_robot_id)
        .bind(rally_result_id),
    )
    .await;

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "rally-view-state".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--rally-result-id".to_string(),
        rally_result_id.to_string(),
        "--require-user-result".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected rally-view-state to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
    assert!(
        stdout.contains("D\tvar rallyViewTest = 1;"),
        "expected result data row in stdout:\n{stdout}"
    );

    let ore_line = find_prefixed_line(&stdout, &format!("O\t{}\t", fixture.ore_id));
    assert!(ore_line[2].ends_with("-ore"));

    let player_zero_line = find_prefixed_line(&stdout, "S\t0\t");
    assert!(player_zero_line[2].ends_with("-robot"));
    assert!(player_zero_line[3].ends_with("-user"));

    let player_one_line = find_prefixed_line(&stdout, "S\t1\t");
    assert!(player_one_line[2].ends_with("-ai"));
    assert!(player_one_line[3].ends_with("-user"));

    let ai_fallback_line = find_prefixed_line(&stdout, "S\t2\t");
    assert!(ai_fallback_line[2].ends_with("-ai"));
    assert!(ai_fallback_line[3].ends_with("-user"));

    let denied = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "rally-view-state".to_string(),
        "--user-id".to_string(),
        fixture.other_user_id.to_string(),
        "--rally-result-id".to_string(),
        rally_result_id.to_string(),
        "--require-user-result".to_string(),
    ]);
    let (denied_stdout, denied_stderr) = output_text(&denied);
    assert!(
        !denied.status.success(),
        "expected inaccessible rally-view-state to fail\nstdout:\n{denied_stdout}\nstderr:\n{denied_stderr}"
    );
    assert!(
        denied_stderr.contains("unknown or inaccessible rally result"),
        "unexpected stderr:\n{denied_stderr}"
    );

    fixture.cleanup(&pool).await;
}

