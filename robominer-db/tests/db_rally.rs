#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{
    CompletedRallyActionRecord, CompletedRallyOreRecord, CompletedRallyParticipantRecord,
    CompletedRallyRecord, cleanup_old_claimed_mining_queue_items_for_robot,
    persist_completed_rally,
};
use robominer_test_support::{
    RallyFixture, insert_ai_robot, insert_area_supply, insert_claimed_mining_queue,
    insert_finished_queue, insert_mining_area, insert_ore, insert_ore_price,
    insert_ore_result_with_depot, insert_robot, insert_row_id, insert_user, insert_user_ore_asset,
    unique_prefix,
};
use serial_test::serial;
use sqlx::Row;

#[tokio::test]
#[serial]
async fn persist_completed_rally_updates_queue_and_score_tables() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
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
                executed_source_code: Some("mine();".to_string()),
                ore_results: vec![CompletedRallyOreRecord {
                    ore_id: fixture.ore_id,
                    amount: 6,
                    depot_amount: 2,
                }],
                action_results: vec![CompletedRallyActionRecord {
                    action_type: 6,
                    amount: 2,
                }],
            }],
        },
    )
    .await
    .expect("persist should succeed")
    .expect("persist should not reject");

    let queue = sqlx::query(
        "SELECT rallyResultId, playerNumber, score, miningEndTime IS NOT NULL AS ended \
         FROM MiningQueue \
         WHERE id = ?",
    )
    .bind(fixture.mining_queue_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load queue row");

    assert_eq!(
        queue.get::<Option<i64>, _>("rallyResultId"),
        Some(rally_result_id)
    );
    assert_eq!(queue.get::<Option<i32>, _>("playerNumber"), Some(0));
    assert!(queue.get::<Option<f64>, _>("score").unwrap_or_default() > 0.0);
    assert_eq!(queue.get::<i8, _>("ended"), 1);

    let executed_source: Option<String> =
        sqlx::query_scalar("SELECT executedSourceCode FROM MiningQueue WHERE id = ?")
            .bind(fixture.mining_queue_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load executed source");
    assert_eq!(executed_source.as_deref(), Some("mine();"));

    let (ore_amount, depot_amount): (i32, i32) = sqlx::query_as(
        "SELECT amount, depotAmount FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?",
    )
    .bind(fixture.mining_queue_id)
    .bind(fixture.ore_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load ore result");
    assert_eq!(ore_amount, 6);
    assert_eq!(depot_amount, 2);

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

    let rejected = persist_completed_rally(
        &pool,
        &CompletedRallyRecord {
            result_data: r#"{"robots":[]}"#.to_string(),
            participants: vec![CompletedRallyParticipantRecord {
                mining_queue_id: fixture.mining_queue_id,
                robot_id: fixture.queued_robot_id,
                mining_area_id: fixture.mining_area_id,
                player_number: 0,
                mining_end_seconds_from_now: 10,
                score: 99.0,
                executed_source_code: Some("mine();".to_string()),
                ore_results: vec![],
                action_results: vec![],
            }],
        },
    )
    .await
    .expect("second persist should not fail at the SQL layer");
    assert!(matches!(
        rejected,
        robominer_db::DbOutcome::Rejected(
            robominer_db::PersistRallyRejection::QueueAlreadyFinished
        )
    ));

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cleanup_old_claimed_mining_queue_items_trims_beyond_retention() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
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

    let summary = cleanup_old_claimed_mining_queue_items_for_robot(&pool, fixture.queued_robot_id)
        .await
        .expect("cleanup should succeed");
    assert_eq!(summary.queues_deleted, 1);

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND claimed = true")
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

#[tokio::test]
#[serial]
async fn list_mining_result_states_for_robot_returns_claimed_only() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('robot-stats-rally')"),
    )
    .await;

    let claimed_id = insert_claimed_mining_queue(
        &pool,
        fixture.mining_area_id,
        fixture.queued_robot_id,
        rally_result_id,
    )
    .await;
    sqlx::query("UPDATE MiningQueue SET score = 12.5 WHERE id = ?")
        .bind(claimed_id)
        .execute(&pool)
        .await
        .expect("set claimed score");
    sqlx::query(
        "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, tax) VALUES (?, ?, 8, 2)",
    )
    .bind(claimed_id)
    .bind(fixture.ore_id)
    .execute(&pool)
    .await
    .expect("insert ore haul");

    // Fixture queue is unclaimed; ensure it stays unclaimed and does not appear.
    sqlx::query("UPDATE MiningQueue SET claimed = false WHERE id = ?")
        .bind(fixture.mining_queue_id)
        .execute(&pool)
        .await
        .expect("keep fixture queue unclaimed");

    let states =
        robominer_db::list_mining_result_states_for_robot(&pool, fixture.queued_robot_id, 5)
            .await
            .expect("list result states");
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].mining_queue_id, claimed_id);
    assert_eq!(states[0].rally_result_id, Some(rally_result_id));
    assert_eq!(states[0].score, 12.5);
    assert_eq!(states[0].total_ore_mined, 8);
    assert_eq!(states[0].total_tax, 2);
    assert_eq!(states[0].total_reward, 6);
    assert_eq!(states[0].mining_area_id, fixture.mining_area_id);
    assert_eq!(states[0].score_ore_target, 30);
    assert!(!states[0].mining_area_name.is_empty());

    let limited =
        robominer_db::list_mining_result_states_for_robot(&pool, fixture.queued_robot_id, 0)
            .await
            .expect("limit 0");
    assert!(limited.is_empty());

    let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
        .bind(claimed_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
        .bind(claimed_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn list_mining_result_states_for_user_returns_newest_claimed_across_robots() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = RallyFixture::create(&pool).await;
    let second_robot_id = insert_robot(
        &pool,
        fixture.user_id,
        &format!("user-results-second-{}", fixture.user_id),
        "mine();",
        1,
    )
    .await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('user-result-states')"),
    )
    .await;

    sqlx::query("UPDATE MiningQueue SET claimed = false WHERE id = ?")
        .bind(fixture.mining_queue_id)
        .execute(&pool)
        .await
        .expect("keep fixture queue unclaimed");

    let mut claimed_ids = Vec::new();
    for (robot_id, seconds_ago) in [
        (fixture.queued_robot_id, 600),
        (fixture.queued_robot_id, 500),
        (fixture.queued_robot_id, 400),
        (second_robot_id, 300),
        (second_robot_id, 200),
        (second_robot_id, 100),
    ] {
        let queue_id =
            insert_claimed_mining_queue(&pool, fixture.mining_area_id, robot_id, rally_result_id)
                .await;
        sqlx::query(
            "UPDATE MiningQueue SET miningEndTime = TIMESTAMPADD(SECOND, ?, NOW()) WHERE id = ?",
        )
        .bind(-seconds_ago)
        .bind(queue_id)
        .execute(&pool)
        .await
        .expect("set claimed queue end time");
        sqlx::query(
            "INSERT INTO MiningOreResult (miningQueueId, oreId, amount, tax) VALUES (?, ?, 8, 2)",
        )
        .bind(queue_id)
        .bind(fixture.ore_id)
        .execute(&pool)
        .await
        .expect("insert ore haul");
        sqlx::query(
            "INSERT INTO RobotActionsDone (miningQueueId, actionType, amount) VALUES (?, 6, 1)",
        )
        .bind(queue_id)
        .execute(&pool)
        .await
        .expect("insert action result");
        claimed_ids.push(queue_id);
    }

    let newest_four = [
        claimed_ids[5],
        claimed_ids[4],
        claimed_ids[3],
        claimed_ids[2],
    ];

    let states = robominer_db::list_mining_result_states_for_user(&pool, fixture.user_id, 4)
        .await
        .expect("list user result states");
    assert_eq!(
        states
            .iter()
            .map(|state| state.mining_queue_id)
            .collect::<Vec<_>>(),
        newest_four
    );
    assert_eq!(states[0].robot_id, second_robot_id);
    assert_eq!(states[3].robot_id, fixture.queued_robot_id);

    let ore_states =
        robominer_db::list_mining_result_ore_states_for_user(&pool, fixture.user_id, 4)
            .await
            .expect("list user ore states");
    let mut ore_queue_ids: Vec<i64> = ore_states.iter().map(|ore| ore.mining_queue_id).collect();
    ore_queue_ids.sort_unstable();
    let mut expected_ore_ids = newest_four;
    expected_ore_ids.sort_unstable();
    assert_eq!(ore_queue_ids, expected_ore_ids);

    let action_states =
        robominer_db::list_mining_result_action_states_for_user(&pool, fixture.user_id, 4)
            .await
            .expect("list user action states");
    let mut action_queue_ids: Vec<i64> = action_states
        .iter()
        .map(|action| action.mining_queue_id)
        .collect();
    action_queue_ids.sort_unstable();
    assert_eq!(action_queue_ids, expected_ore_ids);

    let area_ores = robominer_db::list_mining_result_area_ores_for_user(&pool, fixture.user_id, 4)
        .await
        .expect("list user area ores");
    assert!(
        area_ores
            .iter()
            .any(|ore| ore.mining_area_id == fixture.mining_area_id && ore.ore_id == fixture.ore_id)
    );

    let limited = robominer_db::list_mining_result_states_for_user(&pool, fixture.user_id, 0)
        .await
        .expect("limit 0");
    assert!(limited.is_empty());

    for queue_id in &claimed_ids {
        let _ = sqlx::query("DELETE FROM MiningOreResult WHERE miningQueueId = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM RobotActionsDone WHERE miningQueueId = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
            .bind(queue_id)
            .execute(&pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM RallyResult WHERE id = ?")
        .bind(rally_result_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE id = ?")
        .bind(second_robot_id)
        .execute(&pool)
        .await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn claim_taxes_container_and_depot_ore_separately() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("split-tax");
    let ore_id = insert_ore(&pool, &format!("{prefix}-ore")).await;
    let ore_price_id = insert_ore_price(&pool, &format!("{prefix}-price")).await;
    let user_id = insert_user(&pool, &prefix).await;
    let ai_robot_id = insert_ai_robot(&pool, &format!("{prefix}-ai"), "rotate(90);", 1).await;
    let robot_id = insert_robot(&pool, user_id, &format!("{prefix}-robot"), "mine();", 1).await;
    let mining_area_id = insert_mining_area(&pool, &prefix, ore_price_id, ai_robot_id, 20).await;
    insert_area_supply(&pool, mining_area_id, ore_id, 10, 2).await;
    let queue_id = insert_finished_queue(&pool, mining_area_id, robot_id, -20, -10).await;
    insert_ore_result_with_depot(&pool, queue_id, ore_id, 100, 40).await;
    insert_user_ore_asset(&pool, user_id, ore_id, 0, 1000).await;

    let claimed = robominer_db::claim_user_results(&pool, user_id)
        .await
        .expect("claim should succeed");
    assert_eq!(claimed.claimed_queues, 1);
    assert_eq!(claimed.ore_rewards.len(), 1);
    assert_eq!(claimed.ore_rewards[0].ore_id, ore_id);
    assert_eq!(claimed.ore_rewards[0].reward, 84);

    let tax: i32 =
        sqlx::query_scalar("SELECT tax FROM MiningOreResult WHERE miningQueueId = ? AND oreId = ?")
            .bind(queue_id)
            .bind(ore_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load tax");
    assert_eq!(tax, 16);

    let wallet: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(user_id)
            .bind(ore_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load wallet");
    assert_eq!(wallet, 84);
}
