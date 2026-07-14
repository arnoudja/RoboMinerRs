use robominer_db::{
    CreateProgramSourceRequest, RobotPartTransactionRequest, buy_robot_part,
    create_program_source, sell_robot_part, CancelMiningQueueRequest,
};
use robominer_test_support::{
    QueuedMiningAreaFixture, ShopFixture, insert_user_with_credentials, unique_prefix,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn buy_robot_part_deducts_ore_and_adds_owned_part() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db shop test: ROBOMINER_DATABASE_URL is not set");
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
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db shop test: ROBOMINER_DATABASE_URL is not set");
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
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db shop test: ROBOMINER_DATABASE_URL is not set");
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
async fn cancel_mining_queue_deletes_only_queued_item() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db mining queue test: ROBOMINER_DATABASE_URL is not set");
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
        "test-password",
    )
    .await;
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;

    robominer_db::cancel_mining_queue(
        &pool,
        CancelMiningQueueRequest {
            user_id,
            mining_queue_id: fixture.queued_queue_id,
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
async fn create_program_source_rejects_empty_source_name() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-program");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;

    let rejection = create_program_source(
        &pool,
        CreateProgramSourceRequest {
            user_id,
            source_name: String::new(),
            source_code: "mine();".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect_err("empty source name should reject");

    assert_eq!(
        rejection,
        robominer_db::ProgramSourceWriteRejection::EmptySourceName
    );

    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn create_program_source_inserts_verifiable_row() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-program");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password",
    )
    .await;

    let created = create_program_source(
        &pool,
        CreateProgramSourceRequest {
            user_id,
            source_name: format!("{prefix}-source"),
            source_code: "move(1);".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect("create should succeed");

    let (source_name, verified): (String, bool) = sqlx::query_as(
        "SELECT sourceName, verified FROM ProgramSource WHERE id = ?",
    )
    .bind(created.program_source_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load created program source");
    assert_eq!(source_name, format!("{prefix}-source"));
    assert!(!verified);

    let _ = sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
        .bind(created.program_source_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
