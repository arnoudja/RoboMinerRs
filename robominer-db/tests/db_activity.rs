#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{
    list_activity_rally_participants_for_queues, list_activity_recent_rally_feed,
    list_activity_recent_users, rally_view_metadata, rally_view_state,
};
use robominer_test_support::{CancelMiningQueueFixture, insert_row_id};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn list_activity_recent_rally_feed_filters_by_user_and_area() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = CancelMiningQueueFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('db-activity-feed')"),
    )
    .await;
    fixture.rally_result_id.set(Some(rally_result_id));

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

    sqlx::query("UPDATE User SET lastLoginTime = TIMESTAMPADD(DAY, 1, NOW()) WHERE id = ?")
        .bind(fixture.user_id)
        .execute(&pool)
        .await
        .expect("failed to update recent user login time");

    let recent_users = list_activity_recent_users(&pool, 10)
        .await
        .expect("recent users should load");
    assert!(
        recent_users
            .iter()
            .any(|user| user.user_id == fixture.user_id),
        "expected fixture user in recent users"
    );

    let (all_rallies, _) = list_activity_recent_rally_feed(&pool, None, None, 10)
        .await
        .expect("recent rally feed should load");
    assert!(
        all_rallies
            .iter()
            .any(|rally| rally.mining_queue_id == player_zero_queue_id),
        "expected seeded rally in unfiltered feed"
    );

    let (user_rallies, _) = list_activity_recent_rally_feed(&pool, Some(fixture.user_id), None, 10)
        .await
        .expect("user-filtered rally feed should load");
    assert!(
        user_rallies
            .iter()
            .any(|rally| rally.mining_queue_id == player_zero_queue_id),
        "expected seeded rally for participating user"
    );

    let (area_rallies, _) =
        list_activity_recent_rally_feed(&pool, None, Some(fixture.mining_area_id), 10)
            .await
            .expect("area-filtered rally feed should load");
    assert_eq!(area_rallies.len(), 1);
    assert_eq!(area_rallies[0].mining_queue_id, player_zero_queue_id);

    // AI opponents live in AIRobot and are not MiningQueue rows; only other players appear here.
    let participants = list_activity_rally_participants_for_queues(&pool, &[player_zero_queue_id])
        .await
        .expect("participants should load");
    assert!(participants.is_empty());

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn rally_view_state_requires_claimed_viewer_result_when_requested() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = CancelMiningQueueFixture::create(&pool).await;
    let rally_result_id = insert_row_id(
        &pool,
        sqlx::query("INSERT INTO RallyResult (resultData) VALUES ('viewer-state')"),
    )
    .await;
    fixture.rally_result_id.set(Some(rally_result_id));

    insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningQueue \
             (miningAreaId, robotId, rallyResultId, playerNumber, miningEndTime, claimed) \
             VALUES (?, ?, ?, 0, TIMESTAMPADD(SECOND, -10, NOW()), true)",
        )
        .bind(fixture.mining_area_id)
        .bind(fixture.robot_id)
        .bind(rally_result_id),
    )
    .await;

    let open_view = rally_view_state(&pool, fixture.user_id, rally_result_id, false)
        .await
        .expect("open rally view should load");
    let open_view = open_view.expect("open view present");
    assert!(open_view.ai_robot_name.ends_with("-ai"));
    assert_eq!(open_view.ai_username, "AI");

    let claimed_only = rally_view_state(&pool, fixture.user_id, rally_result_id, true)
        .await
        .expect("claimed-only rally view should load");
    assert!(claimed_only.is_some());

    let metadata = rally_view_metadata(&pool, fixture.user_id, rally_result_id, true)
        .await
        .expect("rally metadata should load")
        .expect("metadata should exist for claimed viewer");
    assert!(metadata.viewer_result_claimed);
    assert_eq!(metadata.viewer_player_number, Some(0));

    fixture.cleanup(&pool).await;
}
