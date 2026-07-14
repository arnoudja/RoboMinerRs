mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn activity_states_report_recent_users_and_rallies() {
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
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('activity-test-rally')"),
    )
    .await;
    fixture.rally_result_id.set(Some(rally_result_id));

    sqlx::query("UPDATE User SET lastLoginTime = TIMESTAMPADD(DAY, 1, NOW()) WHERE id = ?")
        .bind(fixture.user_id)
        .execute(&pool)
        .await
        .expect("failed to update recent user login time");

    let player_zero_queue_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime) \
             VALUES (?, ?, ?, 0, TIMESTAMPADD(SECOND, -10, NOW()))",
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
        database_url,
        "activity-states".to_string(),
        "--user-id".to_string(),
        fixture.user_id.to_string(),
        "--max-users".to_string(),
        "10".to_string(),
        "--max-rallies".to_string(),
        "10".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected activity-states to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");

    let user_line = find_prefixed_line(&stdout, &format!("U\t{}\t", fixture.user_id));
    assert!(user_line[2].ends_with("-user"));
    user_line[3]
        .parse::<i64>()
        .expect("last login time should parse");

    let rally_line = find_prefixed_line(&stdout, &format!("R\t{player_zero_queue_id}\t"));
    assert_eq!(rally_line[2], rally_result_id.to_string());
    assert!(rally_line[3].ends_with("-area"));
    assert!(rally_line[4].ends_with("-robot"));
    assert!(rally_line[5].ends_with("-user"));

    let participant_line = find_prefixed_line(&stdout, &format!("P\t{player_zero_queue_id}\t1\t"));
    assert!(participant_line[3].ends_with("-ai"));
    assert!(participant_line[4].ends_with("-user"));

    fixture.cleanup(&pool).await;
}
