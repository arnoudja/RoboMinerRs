#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{get_request, server_config};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn health_reports_ok_when_database_and_migrations_are_ready() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let config = server_config(pool);

    let response = route(&get_request("/health", None), &config).await;

    assert_eq!(response.status, 200, "health should be ready");
    let body = String::from_utf8_lossy(&response.body);
    assert!(body.starts_with("ok\n"), "body={body}");
    assert!(body.contains("database=ok"), "body={body}");
    assert!(body.contains("migrations=ok"), "body={body}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn health_ready_reports_ok_when_database_and_migrations_are_ready() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let config = server_config(pool);

    let response = route(&get_request("/health/ready", None), &config).await;
    assert_eq!(response.status, 200, "ready should succeed");
    let body = String::from_utf8_lossy(&response.body);
    assert!(body.starts_with("ok\n"), "body={body}");
    assert!(body.contains("database=ok"), "body={body}");

    let live = route(&get_request("/health/live", None), &config).await;
    assert_eq!(live.status, 200, "live should succeed");
}
