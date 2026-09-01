#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::{IdleMiningAreaFixture, QueuedMiningAreaFixture};
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request_query,
    login_with_credentials, post_request, post_request_query, post_request_without_csrf,
    response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_add_post_inserts_queue_item() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-add");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = IdleMiningAreaFixture::create(&pool, user_id, 25).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "add".to_string());
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
    let body = response_body(&response);

    assert_eq!(response.status, 200, "mining queue page should render");
    assert!(
        body.contains("mining-queue-run-active") || body.contains(&fixture.inner.area_name),
        "expected mining run after add in page body:\n{body}"
    );

    let queue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND miningAreaId = ?",
    )
    .bind(fixture.inner.robot_id)
    .bind(fixture.inner.mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count mining queue rows");
    assert_eq!(queue_count, 1);

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_fill_post_inserts_multiple_queue_items() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fill");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = IdleMiningAreaFixture::create(&pool, user_id, 100).await;
    sqlx::query("UPDATE User SET miningQueueSize = 3 WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to expand mining queue size for fill test");
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "fill".to_string());
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
    let body = response_body(&response);

    assert_eq!(response.status, 200, "mining queue fill should render");
    assert!(
        body.contains(&fixture.inner.area_name),
        "expected filled queue runs in page body:\n{body}"
    );

    let queue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND miningAreaId = ?",
    )
    .bind(fixture.inner.robot_id)
    .bind(fixture.inner.mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count mining queue rows");
    assert!(
        queue_count >= 2,
        "expected fill to enqueue multiple runs, got {queue_count}"
    );

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_fragment_post_remove_without_csrf_is_rejected() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fragment-remove-csrf");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    assert_eq!(
        login_response.status, 302,
        "login should redirect after successful authentication"
    );
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("fragment".to_string(), "queue".to_string());

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "remove".to_string());
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

    let mut missing_request =
        post_request_without_csrf("/miningQueue", form.clone(), Some(&cookie));
    missing_request.query = query.clone();
    let missing = route(&missing_request, &config).await;
    assert_eq!(missing.status, 403);
    assert!(
        response_body(&missing).contains("CSRF"),
        "expected CSRF rejection message"
    );

    form.insert("csrfToken".to_string(), "not-a-valid-token".to_string());
    let mut forged_request = post_request_without_csrf("/miningQueue", form, Some(&cookie));
    forged_request.query = query;
    let forged = route(&forged_request, &config).await;
    assert_eq!(forged.status, 403);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.queued_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count mining queue rows");
    assert_eq!(remaining, 1, "CSRF failure must not cancel the queued run");

    fixture.inner.cleanup(&pool, true).await;
}

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_fragment_get_returns_dynamic_sections_only() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fragment-get");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("fragment".to_string(), "queue".to_string());
    query.insert(
        "infoMiningAreaId".to_string(),
        fixture.inner.mining_area_id.to_string(),
    );
    query.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );

    let response = route(
        &get_request_query("/miningQueue", query, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "fragment GET should succeed");
    assert!(
        body.contains(r#"id="mining-queue-fragment""#),
        "expected fragment wrapper:\n{body}"
    );
    assert!(
        body.contains(r#"id="mining-queue-robots-fragment""#),
        "expected robot deck fragment:\n{body}"
    );
    assert!(
        body.contains("app-shell-hud"),
        "expected HUD markup in fragment:\n{body}"
    );
    assert!(
        !body.contains("mining-queue-inspector"),
        "fragment should omit inspector:\n{body}"
    );
    assert!(
        !body.contains("<!DOCTYPE html>"),
        "fragment should omit full page layout:\n{body}"
    );

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_fragment_post_remove_updates_robot_deck() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fragment-remove");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("fragment".to_string(), "queue".to_string());

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "remove".to_string());
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

    let response = route(
        &post_request_query("/miningQueue", query, form, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "fragment POST should succeed");
    assert!(
        body.contains(r#"id="mining-queue-fragment""#),
        "expected fragment wrapper:\n{body}"
    );
    assert!(
        !body.contains("mining-queue-run-queued"),
        "expected queued run to be removed from fragment:\n{body}"
    );
    assert!(
        !body.contains("<!DOCTYPE html>"),
        "fragment should omit full page layout:\n{body}"
    );

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM MiningQueue WHERE id = ?")
        .bind(fixture.queued_queue_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count mining queue rows");
    assert_eq!(remaining, 0);

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
async fn mining_queue_fragment_post_add_inserts_queue_item() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fragment-add");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = IdleMiningAreaFixture::create(&pool, user_id, 25).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("fragment".to_string(), "queue".to_string());

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "add".to_string());
    form.insert("robotId".to_string(), fixture.inner.robot_id.to_string());
    form.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );
    form.insert(
        "infoMiningAreaId".to_string(),
        fixture.inner.mining_area_id.to_string(),
    );

    let response = route(
        &post_request_query("/miningQueue", query, form, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "fragment POST add should succeed");
    assert!(
        body.contains(r#"id="mining-queue-fragment""#),
        "expected fragment wrapper:\n{body}"
    );
    assert!(
        body.contains("mining-queue-run-active") || body.contains(&fixture.inner.area_name),
        "expected active run in fragment after add:\n{body}"
    );

    let queue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM MiningQueue WHERE robotId = ? AND miningAreaId = ?",
    )
    .bind(fixture.inner.robot_id)
    .bind(fixture.inner.mining_area_id)
    .fetch_one(&pool)
    .await
    .expect("failed to count mining queue rows");
    assert_eq!(
        queue_count, 1,
        "expected one queue item after fragment add POST"
    );

    fixture.inner.cleanup(&pool, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_fragment_post_without_csrf_is_rejected() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-fragment-csrf");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = IdleMiningAreaFixture::create(&pool, user_id, 25).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("fragment".to_string(), "queue".to_string());

    let mut form = HashMap::new();
    form.insert("submitType".to_string(), "add".to_string());
    form.insert("robotId".to_string(), fixture.inner.robot_id.to_string());
    form.insert(
        format!("miningArea{}", fixture.inner.robot_id),
        fixture.inner.mining_area_id.to_string(),
    );

    let mut request = post_request_without_csrf("/miningQueue", form, Some(&cookie));
    request.query = query;
    let response = route(&request, &config).await;

    assert_eq!(response.status, 403);
    assert!(
        response_body(&response).contains("CSRF"),
        "expected CSRF rejection for fragment POST"
    );

    fixture.inner.cleanup(&pool, false).await;
}
