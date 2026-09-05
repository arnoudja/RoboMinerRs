#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::QueuedMiningAreaFixture;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_clear_all_post_deletes_queued_items() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-clear-all");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let second_queued = robominer_test_support::insert_mining_queue(
        &pool,
        fixture.inner.mining_area_id,
        fixture.inner.robot_id,
    )
    .await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "clear".to_string());
    form.insert("clearMode".to_string(), "all".to_string());
    form.insert("robotId".to_string(), fixture.inner.robot_id.to_string());
    form.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );
    form.insert(
        "infoMiningAreaId".to_string(),
        fixture.inner.mining_area_id.to_string(),
    );

    let response = route(&post_request("/miningQueue", form, Some(&cookie)), &config).await;
    assert_eq!(response.status, 200, "mining queue page should render");

    let queued_remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND id IN (?, ?)")
            .bind(fixture.inner.robot_id)
            .bind(fixture.queued_queue_id)
            .bind(second_queued)
            .fetch_one(&pool)
            .await
            .expect("failed to count queued rows");
    assert_eq!(queued_remaining, 0);

    let active_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.active_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count active mining queue row");
    assert_eq!(active_remaining, 1);

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_clear_selected_post_deletes_only_selected_queued_item() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-clear-sel");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let second_queued = robominer_test_support::insert_mining_queue(
        &pool,
        fixture.inner.mining_area_id,
        fixture.inner.robot_id,
    )
    .await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "clear".to_string());
    form.insert("clearMode".to_string(), "all".to_string());
    form.insert("robotId".to_string(), fixture.inner.robot_id.to_string());
    form.insert(
        "selectedQueueItemId".to_string(),
        fixture.queued_queue_id.to_string(),
    );
    form.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );
    form.insert(
        "infoMiningAreaId".to_string(),
        fixture.inner.mining_area_id.to_string(),
    );

    let response = route(&post_request("/miningQueue", form, Some(&cookie)), &config).await;
    assert_eq!(response.status, 200, "mining queue page should render");

    let selected_remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
            .bind(fixture.queued_queue_id)
            .fetch_one(&pool)
            .await
            .expect("failed to count selected queued row");
    assert_eq!(selected_remaining, 0);

    let other_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(second_queued)
        .fetch_one(&pool)
        .await
        .expect("failed to count other queued row");
    assert_eq!(other_remaining, 1);

    let active_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.active_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count active mining queue row");
    assert_eq!(active_remaining, 1);

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_clear_safe_skips_overflow_and_keeps_later_safe_items() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-clear-safe");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;

    // Expensive queued item (cost 5) then cheap (cost 1). Wallet 8/10: skip expensive, clear cheap.
    sqlx::query("UPDATE OrePriceAmount SET amount = 5 WHERE orePriceId = ? AND oreId = ?")
        .bind(fixture.inner.ore_price_id)
        .bind(fixture.inner.ore_id)
        .execute(&pool)
        .await
        .expect("failed to raise expensive area cost");
    robominer_test_support::insert_user_ore_asset(&pool, user_id, fixture.inner.ore_id, 8, 10)
        .await;

    let cheap_ore_price_id = robominer_test_support::insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePrice (description) VALUES (?)")
            .bind(format!("{prefix}-cheap-price")),
    )
    .await;
    robominer_test_support::insert_row_id(
        &pool,
        sqlx::query("INSERT INTO OrePriceAmount (orePriceId, oreId, amount) VALUES (?, ?, ?)")
            .bind(cheap_ore_price_id)
            .bind(fixture.inner.ore_id)
            .bind(1),
    )
    .await;
    let cheap_area_id = robominer_test_support::insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO MiningArea \
             (areaName, orePriceId, sizeX, sizeY, maxMoves, miningTime, taxRate, aiRobotId) \
             VALUES (?, ?, 4, 4, 1, 10, 0, ?)",
        )
        .bind(format!("{prefix}-cheap-area"))
        .bind(cheap_ore_price_id)
        .bind(fixture.inner.ai_robot_id),
    )
    .await;
    sqlx::query("INSERT INTO UserMiningArea (userId, miningAreaId) VALUES (?, ?)")
        .bind(user_id)
        .bind(cheap_area_id)
        .execute(&pool)
        .await
        .expect("failed to grant cheap mining area");
    let cheap_queued =
        robominer_test_support::insert_mining_queue(&pool, cheap_area_id, fixture.inner.robot_id)
            .await;

    let config = server_config(pool.clone());
    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "clear".to_string());
    form.insert("clearMode".to_string(), "safe".to_string());
    form.insert("robotId".to_string(), fixture.inner.robot_id.to_string());
    form.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );
    form.insert(
        "infoMiningAreaId".to_string(),
        fixture.inner.mining_area_id.to_string(),
    );

    let response = route(&post_request("/miningQueue", form, Some(&cookie)), &config).await;
    assert_eq!(response.status, 200, "mining queue page should render");

    let expensive_remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
            .bind(fixture.queued_queue_id)
            .fetch_one(&pool)
            .await
            .expect("count expensive queued");
    assert_eq!(
        expensive_remaining, 1,
        "unsafe expensive queued item should be skipped"
    );

    let cheap_remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(cheap_queued)
        .fetch_one(&pool)
        .await
        .expect("count cheap queued");
    assert_eq!(
        cheap_remaining, 0,
        "later safe cheap item should be cleared"
    );

    let amount: i32 =
        sqlx::query_scalar("SELECT amount FROM UserOreAsset WHERE userId = ? AND oreId = ?")
            .bind(user_id)
            .bind(fixture.inner.ore_id)
            .fetch_one(&pool)
            .await
            .expect("ore amount");
    assert_eq!(amount, 9, "only the cheap cost of 1 should be refunded");

    let _ = sqlx::query("DELETE FROM MiningQueue WHERE id = ?")
        .bind(cheap_queued)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM UserMiningArea WHERE userId = ? AND miningAreaId = ?")
        .bind(user_id)
        .bind(cheap_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM MiningArea WHERE id = ?")
        .bind(cheap_area_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePriceAmount WHERE orePriceId = ?")
        .bind(cheap_ore_price_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM OrePrice WHERE id = ?")
        .bind(cheap_ore_price_id)
        .execute(&pool)
        .await;
    fixture.inner.cleanup(&pool, true).await;
}
