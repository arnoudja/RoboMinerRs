use robominer_db::{
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord, cleanup_old_claimed_mining_queue_items_for_robot, persist_completed_rally,
};
use robominer_test_support::{RallyFixture, insert_claimed_mining_queue, insert_row_id};
use serial_test::serial;
use sqlx::Row;

#[tokio::test]
#[serial]
async fn persist_completed_rally_updates_queue_and_score_tables() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db rally test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;

    let rally_result_id = persist_completed_rally(
        &pool,
        &CompletedRallyRecord {
            result_data: r#"{"robots":[]}"#.to_string(),
            participants: vec![CompletedRallyParticipantRecord {
                mining_queue_id: fixture.mining_queue_id,
                robot_id: fixture.queued_robot_id,
                mining_area_id: fixture.mining_area_id,
                player_number: 0,
                mining_end_seconds_from_now: 10,
                score: 88.0,
                ore_results: vec![CompletedRallyOreRecord {
                    ore_id: fixture.ore_id,
                    amount: 6,
                }],
                action_results: vec![CompletedRallyActionRecord {
                    action_type: 6,
                    amount: 2,
                }],
            }],
        },
    )
    .await
    .expect("persist should succeed");

    let queue = sqlx::query(
        "SELECT rallyResultId, playerNumber, score, miningEndTime IS NOT NULL AS ended \
         FROM MiningQueue \
         WHERE id = ?",
    )
    .bind(fixture.mining_queue_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load queue row");

    assert_eq!(queue.get::<Option<i64>, _>("rallyResultId"), Some(rally_result_id));
    assert_eq!(queue.get::<Option<i32>, _>("playerNumber"), Some(0));
    assert!(queue.get::<Option<f64>, _>("score").unwrap_or_default() > 0.0);
    assert_eq!(queue.get::<i8, _>("ended"), 1);

    let ore_amount: i32 = sqlx::query_scalar(
        "SELECT amount FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?",
    )
    .bind(fixture.mining_queue_id)
    .bind(fixture.ore_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load ore result");
    assert_eq!(ore_amount, 6);

    let action_amount: i32 = sqlx::query_scalar(
        "SELECT amount FROM RobotActionsDone WHERE miningQueueId = ? AND actionType = 6",
    )
    .bind(fixture.mining_queue_id)
    .fetch_optional(&pool)
    .await
    .expect("failed to load action result")
    .unwrap_or(0);
    assert_eq!(action_amount, 2);

    let (total_runs, smoothed_score): (i32, f64) = sqlx::query_as(
        "SELECT totalRuns, score FROM RobotMiningAreaScore WHERE robotId = ? AND miningAreaId = ?",
    )
    .bind(fixture.queued_robot_id)
    .bind(fixture.mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load robot score");
    assert_eq!(total_runs, 1);
    assert!(smoothed_score > 0.0);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cleanup_old_claimed_mining_queue_items_trims_beyond_retention() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db rally test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('cleanup-test-rally')"),
    )
    .await;

    let mut queue_ids = Vec::new();
    for _ in 0..13 {
        queue_ids.push(
            insert_claimed_mining_queue(
                &pool,
                fixture.mining_area_id,
                fixture.queued_robot_id,
                rally_result_id,
            )
            .await,
        );
    }

    let summary =
        cleanup_old_claimed_mining_queue_items_for_robot(&pool, fixture.queued_robot_id)
            .await
            .expect("cleanup should succeed");
    assert_eq!(summary.queues_deleted, 1);

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND claimed = true",
    )
    .bind(fixture.queued_robot_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count remaining claimed queues");
    assert_eq!(remaining, 12);

    let oldest_id = queue_ids[0];
    let oldest_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(oldest_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count oldest queue");
    assert_eq!(oldest_remaining, 0);

    for queue_id in queue_ids {
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}
