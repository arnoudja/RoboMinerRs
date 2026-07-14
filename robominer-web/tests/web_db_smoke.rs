mod support;

use robominer_web::test_support::route;
use support::{
    WebSmokeFixture, cookie_header, ensure_session_configured, response_body, server_config,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_db_smoke_suite() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping web DB smoke tests: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let database_url = std::env::var("ROBOMINER_DATABASE_URL").expect("database url");
    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = WebSmokeFixture::create(&pool).await;
    let config = server_config(pool.clone());

    let login_response = fixture.login(&config);
    assert_eq!(
        login_response.status, 302,
        "login should redirect after success"
    );
    assert!(
        login_response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value == "miningQueue"),
        "login should redirect to mining queue"
    );

    let cookie = cookie_header(&login_response);
    assert!(
        cookie.contains("robominer_session="),
        "login should mint a session cookie"
    );

    let queue_response = fixture.mining_queue_page(&config, &cookie);
    let body = response_body(&queue_response);

    assert_eq!(
        queue_response.status, 200,
        "mining queue page should render"
    );
    assert!(
        body.contains(&fixture.robot_name),
        "expected robot name in queue page body:\n{body}"
    );
    assert!(
        body.contains(&fixture.area_name),
        "expected mining area name in queue page body:\n{body}"
    );
    assert!(
        body.contains("mining-queue-run-active"),
        "expected a current mining run in queue page body:\n{body}"
    );

    let root_response = route(&support::get_request("/", Some(&cookie)), &config);
    assert_eq!(root_response.status, 302);
    assert!(
        root_response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value == "miningQueue")
    );

    fixture.cleanup(&pool).await;
}
