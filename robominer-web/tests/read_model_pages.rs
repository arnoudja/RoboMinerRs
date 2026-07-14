mod support;

use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    WebSmokeFixture, cookie_header, ensure_session_configured, response_body, server_config,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn read_model_pages_render_with_seeded_data() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping read-model web tests: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let database_url = std::env::var("ROBOMINER_DATABASE_URL").expect("database url");
    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = WebSmokeFixture::create(&pool).await;
    let config = server_config(pool.clone());
    let cookie = cookie_header(&fixture.login(&config));

    for (path, marker) in [
        ("/miningResults", "mining-results-page"),
        ("/leaderboard", "leaderboard-page"),
        ("/miningAreaOverview", "mining-area-atlas-title"),
        ("/activity", "activity-page"),
    ] {
        let response = route(&support::get_request(path, Some(&cookie)), &config);
        let body = response_body(&response);

        assert_eq!(response.status, 200, "{path} should render");
        assert!(
            body.contains(marker),
            "expected {path} to contain {marker}:\n{body}"
        );
    }

    fixture.cleanup(&pool).await;
}
