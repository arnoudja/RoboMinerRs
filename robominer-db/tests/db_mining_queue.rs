#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{DbOutcome, EnqueueMiningRejection, EnqueueMiningRequest, enqueue_mining};
use robominer_test_support::{EnqueueMiningFixture, unique_prefix};
use serial_test::serial;

async fn waiting_queue_count(pool: &sqlx::MySqlPool, robot_id: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) \
         FROM MiningQueue \
         WHERE robotId = ? \
           AND (miningEndTime IS NULL OR miningEndTime > NOW())",
    )
    .bind(robot_id)
    .fetch_one(pool)
    .await
    .expect("failed to count waiting queue items")
}

#[tokio::test]
#[serial]
async fn enqueue_mining_rejects_when_queue_is_full() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 1, 25, 4, true).await;

    enqueue_mining(
        &pool,
        EnqueueMiningRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            mining_area_id: fixture.mining_area_id,
            fill: false,
        },
    )
    .await
    .expect("first enqueue should not fail at sql layer")
    .expect("first enqueue should succeed");

    let rejection = enqueue_mining(
        &pool,
        EnqueueMiningRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            mining_area_id: fixture.mining_area_id,
            fill: false,
        },
    )
    .await
    .expect("second enqueue should not fail at sql layer")
    .expect_err("second enqueue should reject a full queue");

    assert_eq!(rejection, EnqueueMiningRejection::QueueFull);
    assert_eq!(waiting_queue_count(&pool, fixture.robot_id).await, 1);

    fixture.cleanup(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[serial]
async fn enqueue_mining_concurrent_add_respects_queue_limit() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = EnqueueMiningFixture::create(&pool, 2, 100, 1, true).await;
    let prefix = unique_prefix("rust-db-enqueue-race");

    let mut handles = Vec::with_capacity(6);
    for index in 0..6 {
        let pool = pool.clone();
        let request = EnqueueMiningRequest {
            user_id: fixture.user_id,
            robot_id: fixture.robot_id,
            mining_area_id: fixture.mining_area_id,
            fill: false,
        };
        handles.push(tokio::spawn(async move {
            let result = enqueue_mining(&pool, request)
                .await
                .expect("enqueue should not fail at sql layer");
            (index, result)
        }));
    }

    let mut successes = 0;
    let mut rejections = 0;
    for handle in handles {
        let (index, result) = handle.await.expect("enqueue task should complete");
        match result {
            DbOutcome::Success(_) => successes += 1,
            DbOutcome::Rejected(EnqueueMiningRejection::QueueFull) => rejections += 1,
            DbOutcome::Rejected(other) => {
                panic!("unexpected enqueue rejection from task {index}: {other:?}")
            }
        }
    }

    let waiting_count = waiting_queue_count(&pool, fixture.robot_id).await;
    assert_eq!(
        waiting_count, 2,
        "concurrent enqueues must not exceed the per-robot queue limit"
    );
    assert_eq!(
        successes, 2,
        "exactly two concurrent enqueues should succeed for queue size 2"
    );
    assert_eq!(
        rejections, 4,
        "remaining concurrent enqueues should reject with QueueFull"
    );

    eprintln!("{prefix}: concurrent enqueue race test passed");

    fixture.cleanup(&pool).await;
}
