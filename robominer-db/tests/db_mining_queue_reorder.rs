#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{
    CancelMiningQueueRequest, DbOutcome, EnqueueMiningRequest, MiningQueueMoveDirection,
    MoveMiningQueueRequest, ReorderMiningQueueRejection, ReorderMiningQueueRequest,
    cancel_mining_queue, enqueue_mining, list_mining_queue_page_items,
    list_next_mining_rally_queue_for_area, move_mining_queue_item, reorder_mining_queue,
};
use robominer_test_support::EnqueueMiningFixture;
use serial_test::serial;

async fn enqueue_runs(pool: &sqlx::MySqlPool, fixture: &EnqueueMiningFixture, count: usize) {
    for _ in 0..count {
        enqueue_mining(
            pool,
            EnqueueMiningRequest {
                user_id: fixture.user_id,
                robot_id: fixture.robot_id,
                mining_area_id: fixture.mining_area_id,
                fill: false,
            },
        )
        .await
        .expect("enqueue should not fail at sql layer")
        .expect("enqueue should succeed");
    }
}

async fn unfinished_ids(pool: &sqlx::MySqlPool, user_id: i64) -> Vec<i64> {
    list_mining_queue_page_items(pool, user_id)
        .await
        .expect("list queue items")
        .into_iter()
        .map(|item| item.mining_queue_id)
        .collect()
}

async fn queue_order(pool: &sqlx::MySqlPool, mining_queue_id: i64) -> i64 {
    sqlx::query_scalar("SELECT queueOrder FROM MiningQueue WHERE id = ?")
        .bind(mining_queue_id)
        .fetch_one(pool)
        .await
        .expect("queueOrder should load")
}

#[tokio::test]
#[serial]
async fn enqueue_mining_sets_queue_order_to_id() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 2, 25, 4, true).await;
    enqueue_runs(&pool, &fixture, 1).await;

    let ids = unfinished_ids(&pool, fixture.user_id).await;
    assert_eq!(ids.len(), 1);
    assert_eq!(queue_order(&pool, ids[0]).await, ids[0]);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn reorder_mining_queue_swaps_queued_items_and_keeps_head() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 3, 40, 4, true).await;
    enqueue_runs(&pool, &fixture, 3).await;

    let ids = unfinished_ids(&pool, fixture.user_id).await;
    assert_eq!(ids.len(), 3);
    let head = ids[0];
    let first_queued = ids[1];
    let second_queued = ids[2];

    reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            ordered_queue_ids: vec![second_queued, first_queued],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect("reorder should succeed");

    assert_eq!(
        unfinished_ids(&pool, fixture.user_id).await,
        vec![head, second_queued, first_queued]
    );

    let rally_heads = list_next_mining_rally_queue_for_area(&pool, fixture.mining_area_id)
        .await
        .expect("list rally heads");
    assert!(
        rally_heads
            .iter()
            .any(|row| row.queue.id == head && row.queue.robot_id == fixture.robot_id),
        "rally claim must still select the original head"
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn move_mining_queue_item_swaps_adjacent_queued_rows() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 3, 40, 4, true).await;
    enqueue_runs(&pool, &fixture, 3).await;

    let ids = unfinished_ids(&pool, fixture.user_id).await;
    let head = ids[0];
    let first_queued = ids[1];
    let second_queued = ids[2];

    move_mining_queue_item(
        &pool,
        MoveMiningQueueRequest {
            user_id: fixture.user_id,
            mining_queue_id: first_queued,
            direction: MiningQueueMoveDirection::Down,
        },
    )
    .await
    .expect("move should not fail at sql layer")
    .expect("move down should succeed");

    assert_eq!(
        unfinished_ids(&pool, fixture.user_id).await,
        vec![head, second_queued, first_queued]
    );

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn reorder_mining_queue_rejects_moving_the_head() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 3, 40, 4, true).await;
    enqueue_runs(&pool, &fixture, 3).await;

    let ids = unfinished_ids(&pool, fixture.user_id).await;
    let rejection = reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            ordered_queue_ids: vec![ids[0], ids[2]],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect_err("including the head must reject");
    assert_eq!(rejection, ReorderMiningQueueRejection::NotReorderable);

    let move_rejection = move_mining_queue_item(
        &pool,
        MoveMiningQueueRequest {
            user_id: fixture.user_id,
            mining_queue_id: ids[0],
            direction: MiningQueueMoveDirection::Down,
        },
    )
    .await
    .expect("move should not fail at sql layer")
    .expect_err("moving the head must reject");
    assert_eq!(move_rejection, ReorderMiningQueueRejection::NotReorderable);

    assert_eq!(unfinished_ids(&pool, fixture.user_id).await, ids);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn reorder_mining_queue_rejects_unknown_robot_and_foreign_queue() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 3, 40, 4, true).await;
    enqueue_runs(&pool, &fixture, 3).await;
    let ids = unfinished_ids(&pool, fixture.user_id).await;

    let unknown_robot = reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id + 1_000_000,
            ordered_queue_ids: vec![ids[1], ids[2]],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect_err("unknown robot must reject");
    assert_eq!(unknown_robot, ReorderMiningQueueRejection::UnknownRobot);

    let other_user = reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.other_user_id,
            robot_id: fixture.robot_id,
            ordered_queue_ids: vec![ids[1], ids[2]],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect_err("wrong owner must reject");
    assert_eq!(other_user, ReorderMiningQueueRejection::UnknownRobot);

    let unknown_queue = reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            ordered_queue_ids: vec![ids[1], ids[2] + 1_000_000],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect_err("unknown queue id must reject");
    assert_eq!(unknown_queue, ReorderMiningQueueRejection::UnknownQueue);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn cancel_still_rejects_head_after_queued_reorder() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 3, 40, 4, true).await;
    enqueue_runs(&pool, &fixture, 3).await;
    let ids = unfinished_ids(&pool, fixture.user_id).await;

    reorder_mining_queue(
        &pool,
        ReorderMiningQueueRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            ordered_queue_ids: vec![ids[2], ids[1]],
        },
    )
    .await
    .expect("reorder should not fail at sql layer")
    .expect("reorder should succeed");

    let head_cancel = cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id: fixture.user_id,
            mining_queue_id: ids[0],
            require_refund_fits: false,
        },
    )
    .await
    .expect("cancel should not fail at sql layer");
    assert!(
        matches!(head_cancel, DbOutcome::Rejected(_)),
        "head must stay non-cancelable after reorder"
    );

    cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id: fixture.user_id,
            mining_queue_id: ids[2],
            require_refund_fits: false,
        },
    )
    .await
    .expect("cancel should not fail at sql layer")
    .expect("first queued item after reorder should cancel");

    fixture.cleanup(&pool).await;
}
