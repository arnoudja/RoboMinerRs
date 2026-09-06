#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::CancelMiningQueueRequest;
use robominer_test_support::{
    QueuedMiningAreaFixture, insert_user_ore_asset, insert_user_with_credentials, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn cancel_mining_queue_deletes_only_queued_item() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-cancel-queue");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;

    robominer_db::cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id,
            mining_queue_id: fixture.queued_queue_id,
            require_refund_fits: false,
        },
    )
    .await
    .expect("cancel should not fail at sql layer")
    .expect("queued item should cancel");

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.queued_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count queued row");
    assert_eq!(remaining, 0);

    let active_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.active_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count active row");
    assert_eq!(active_remaining, 1);

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_refunds_ore_cost() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-cancel-refund");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    // Area cost is 1 of the fixture ore (see RobotMiningAreaFixture).
    insert_user_ore_asset(&pool, user_id, fixture.inner.ore_id, 5, 100).await;

    robominer_db::cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id,
            mining_queue_id: fixture.queued_queue_id,
            require_refund_fits: false,
        },
    )
    .await
    .expect("cancel should not fail at sql layer")
    .expect("queued item should cancel");

    let amount: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(user_id)
            .bind(fixture.inner.ore_id)
            .fetch_one(&pool)
            .await
            .expect("ore amount should load");
    assert_eq!(amount, 6, "full area cost of 1 should be refunded");

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_refund_clamps_to_max_allowed() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-cancel-clamp");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    insert_user_ore_asset(&pool, user_id, fixture.inner.ore_id, 10, 10).await;

    robominer_db::cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id,
            mining_queue_id: fixture.queued_queue_id,
            require_refund_fits: false,
        },
    )
    .await
    .expect("cancel should not fail at sql layer")
    .expect("queued item should cancel");

    let amount: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(user_id)
            .bind(fixture.inner.ore_id)
            .fetch_one(&pool)
            .await
            .expect("ore amount should load");
    assert_eq!(amount, 10, "refund must not exceed maxAllowed");

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test]
#[serial]
async fn cancel_mining_queue_require_refund_fits_skips_clamp() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-cancel-fit");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    insert_user_ore_asset(&pool, user_id, fixture.inner.ore_id, 10, 10).await;

    let result = robominer_db::cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id,
            mining_queue_id: fixture.queued_queue_id,
            require_refund_fits: true,
        },
    )
    .await
    .expect("cancel should not fail at sql layer");
    assert_eq!(
        result.into_result(),
        Err(robominer_db::CancelMiningQueueRejection::RefundWouldClamp)
    );

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.queued_queue_id)
        .fetch_one(&pool)
        .await
        .expect("queued row should remain");
    assert_eq!(remaining, 1);

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test]
#[serial]
async fn ore_refund_fits_without_clamp_detects_wallet_cap() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-refund-fit");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    insert_user_ore_asset(&pool, user_id, fixture.inner.ore_id, 10, 10).await;

    let fits =
        robominer_db::ore_refund_fits_without_clamp(&pool, user_id, &[(fixture.inner.ore_id, 1)])
            .await
            .expect("headroom check should not fail");
    assert!(!fits, "refund of 1 into a full wallet should not fit");

    sqlx::query(
        "UPDATE UserOreAsset SET amount = 9, maxAllowed = 10 WHERE userId = ? AND oreId = ?",
    )
    .bind(user_id)
    .bind(fixture.inner.ore_id)
    .execute(&pool)
    .await
    .expect("wallet update should succeed");
    let fits_after =
        robominer_db::ore_refund_fits_without_clamp(&pool, user_id, &[(fixture.inner.ore_id, 1)])
            .await
            .expect("headroom check should not fail");
    assert!(fits_after, "refund of 1 into 9/10 should fit");

    fixture.inner.cleanup(&pool, true).await;
}
