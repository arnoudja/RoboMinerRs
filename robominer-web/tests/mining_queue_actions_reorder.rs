#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::{QueuedMiningAreaFixture, insert_mining_queue};
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, server_config, unique_prefix,
};

async fn unfinished_ids(pool: &robominer_db::MySqlPool, user_id: i64) -> Vec<i64> {
    robominer_db::list_mining_queue_page_items(pool, user_id)
        .await
        .expect("list queue items")
        .into_iter()
        .map(|item| item.mining_queue_id)
        .collect()
}

fn queue_form(robot_id: i64, mining_area_id: i64) -> HashMap<String, String> {
    let mut form = HashMap::new();
    form.insert("robotId".to_string(), robot_id.to_string());
    form.insert(format!("miningArea{robot_id}"), mining_area_id.to_string());
    form.insert("infoMiningAreaId".to_string(), mining_area_id.to_string());
    form
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_move_down_post_swaps_queued_items() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-move");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let third_queued =
        insert_mining_queue(&pool, fixture.inner.mining_area_id, fixture.inner.robot_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = queue_form(fixture.inner.robot_id, fixture.inner.mining_area_id);
    form.insert("moveDown".to_string(), fixture.queued_queue_id.to_string());

    let response = route(&post_request("/miningQueue", form, Some(&cookie)), &config).await;
    assert_eq!(response.status, 200, "mining queue move should render");

    assert_eq!(
        unfinished_ids(&pool, user_id).await,
        vec![
            fixture.active_queue_id,
            third_queued,
            fixture.queued_queue_id
        ]
    );

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_reorder_post_sets_queued_order_without_moving_head() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-reorder");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let third_queued =
        insert_mining_queue(&pool, fixture.inner.mining_area_id, fixture.inner.robot_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = queue_form(fixture.inner.robot_id, fixture.inner.mining_area_id);
    form.insert("submitType".to_string(), "reorder".to_string());
    let mut request = post_request("/miningQueue", form, Some(&cookie));
    request.form_values.insert(
        "orderedQueueItemId".to_string(),
        vec![
            third_queued.to_string(),
            fixture.queued_queue_id.to_string(),
        ],
    );

    let response = route(&request, &config).await;
    assert_eq!(response.status, 200, "mining queue reorder should render");

    assert_eq!(
        unfinished_ids(&pool, user_id).await,
        vec![
            fixture.active_queue_id,
            third_queued,
            fixture.queued_queue_id
        ]
    );

    fixture.inner.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn mining_queue_reorder_post_wins_over_leftover_clear_submit_type() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-queue-reorder-clear");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = QueuedMiningAreaFixture::create(&pool, user_id).await;
    let third_queued =
        insert_mining_queue(&pool, fixture.inner.mining_area_id, fixture.inner.robot_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = queue_form(fixture.inner.robot_id, fixture.inner.mining_area_id);
    form.insert("submitType".to_string(), "clear".to_string());
    let mut request = post_request("/miningQueue", form, Some(&cookie));
    request.form_values.insert(
        "submitType".to_string(),
        vec!["clear".to_string(), "reorder".to_string()],
    );
    request.form_values.insert(
        "orderedQueueItemId".to_string(),
        vec![
            third_queued.to_string(),
            fixture.queued_queue_id.to_string(),
        ],
    );

    let response = route(&request, &config).await;
    assert_eq!(response.status, 200, "mining queue reorder should render");

    assert_eq!(
        unfinished_ids(&pool, user_id).await,
        vec![
            fixture.active_queue_id,
            third_queued,
            fixture.queued_queue_id
        ]
    );

    fixture.inner.cleanup(&pool, true).await;
}
