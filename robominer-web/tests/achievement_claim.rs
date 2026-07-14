mod support;

use std::collections::HashMap;

use robominer_test_support::AchievementScenario;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, login_with_credentials,
    post_request, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_claim_post_applies_rewards() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping achievement claim web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool = robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-achievement");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id = create_user_via_engine(
        &username,
        &format!("{prefix}@example.invalid"),
        &password,
    );
    let fixture = AchievementScenario::attach_to_user(&pool, &prefix, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password);
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "achievementId".to_string(),
        fixture.achievement_id.to_string(),
    );

    let response = route(&post_request("/achievements", form, Some(&cookie)), &config);
    let body = response_body(&response);

    assert_eq!(response.status, 200, "achievements page should render");
    assert!(
        body.contains("Achievement claimed"),
        "expected claim success message in achievements body:\n{body}"
    );

    fixture.assert_claimed(&pool, 17, 3).await;
    fixture.cleanup(&pool, true).await;
}
