mod support;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{get_request, server_config};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn health_reports_ok_when_database_and_migrations_are_ready() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping health web test: ROBOMINER_DATABASE_URL is not set");
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
