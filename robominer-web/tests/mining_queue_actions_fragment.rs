#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::{IdleMiningAreaFixture, QueuedMiningAreaFixture};
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request_query,
    login_with_credentials, post_request_query, post_request_without_csrf, response_body,
    server_config, unique_prefix,
};

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
