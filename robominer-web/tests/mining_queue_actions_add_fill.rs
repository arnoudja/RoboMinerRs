#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::IdleMiningAreaFixture;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, response_body, server_config, unique_prefix,
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
