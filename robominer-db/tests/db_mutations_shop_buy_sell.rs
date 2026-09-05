#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{
    RobotPartTransactionRequest, buy_robot_part, sell_all_unassigned_robot_parts, sell_robot_part,
};
use robominer_test_support::ShopFixture;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn buy_robot_part_deducts_ore_and_adds_owned_part() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 25, 10, 0, false).await;

    let result = buy_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: fixture.user_id,
            robot_part_id: fixture.robot_part_id,
        },
    )
    .await
    .expect("buy should not fail at sql layer")
    .expect("buy should succeed");

    assert_eq!(result.robot_part_id, fixture.robot_part_id);
    fixture.assert_ore_amount(&pool, 15).await;
    fixture.assert_robot_part_total_owned(&pool, Some(1)).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn buy_robot_part_rejects_insufficient_funds() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 5, 10, 0, false).await;

    let rejection = buy_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: fixture.user_id,
            robot_part_id: fixture.robot_part_id,
        },
    )
    .await
    .expect("buy should not fail at sql layer")
    .expect_err("buy should reject insufficient funds");

    assert_eq!(
        rejection,
        robominer_db::RobotPartTransactionRejection::InsufficientFunds
    );
    fixture.assert_ore_amount(&pool, 5).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_robot_part_refunds_half_cost_and_clears_unassigned_stock() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 0, 10, 1, false).await;

    sell_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: fixture.user_id,
            robot_part_id: fixture.robot_part_id,
        },
    )
    .await
    .expect("sell should not fail at sql layer")
    .expect("sell should succeed");

    fixture.assert_ore_amount(&pool, 5).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn buy_robot_part_rejects_unknown_user() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 25, 10, 0, false).await;

    let rejection = buy_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: i64::MAX - 7,
            robot_part_id: fixture.robot_part_id,
        },
    )
    .await
    .expect("buy should not fail at sql layer")
    .expect_err("buy should reject unknown user");

    assert_eq!(
        rejection,
        robominer_db::RobotPartTransactionRejection::UnknownUser
    );
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn buy_robot_part_rejects_unknown_robot_part() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 25, 10, 0, false).await;

    let rejection = buy_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: fixture.user_id,
            robot_part_id: i64::MAX - 7,
        },
    )
    .await
    .expect("buy should not fail at sql layer")
    .expect_err("buy should reject unknown robot part");

    assert_eq!(
        rejection,
        robominer_db::RobotPartTransactionRejection::UnknownRobotPart
    );
    fixture.assert_ore_amount(&pool, 25).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_robot_part_rejects_no_unassigned_stock() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 0, 10, 0, false).await;

    let rejection = sell_robot_part(
        &pool,
        RobotPartTransactionRequest {
            user_id: fixture.user_id,
            robot_part_id: fixture.robot_part_id,
        },
    )
    .await
    .expect("sell should not fail at sql layer")
    .expect_err("sell should reject when no unassigned stock");

    assert_eq!(
        rejection,
        robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart
    );
    fixture.assert_ore_amount(&pool, 0).await;
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_all_unassigned_robot_parts_rejects_when_empty() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 0, 10, 0, false).await;

    let rejection = sell_all_unassigned_robot_parts(&pool, fixture.user_id)
        .await
        .expect("sell-all should not fail at sql layer")
        .expect_err("sell-all should reject when empty");

    assert_eq!(
        rejection,
        robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart
    );
    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn sell_all_unassigned_robot_parts_sells_stock() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ShopFixture::create(&pool, 0, 10, 2, false).await;

    let result = sell_all_unassigned_robot_parts(&pool, fixture.user_id)
        .await
        .expect("sell-all should not fail at sql layer")
        .expect("sell-all should succeed");

    assert_eq!(result.sold_count, 2);
    fixture.assert_ore_amount(&pool, 10).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup(&pool).await;
}
