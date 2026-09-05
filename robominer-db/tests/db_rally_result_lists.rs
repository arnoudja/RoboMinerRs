#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_test_support::{
    RallyFixture, insert_claimed_mining_queue, insert_robot, insert_row_id,
};
use serial_test::serial;

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
