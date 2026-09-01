#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_test_support::AchievementScenario;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request_query,
    login_with_credentials, post_request, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_claim_post_applies_rewards() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-achievement");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = AchievementScenario::attach_to_user(&pool, &prefix, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "achievementId".to_string(),
        fixture.achievement_id.to_string(),
    );

    let response = route(&post_request("/achievements", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "achievements page should render");
    assert!(
        body.contains("Achievement claimed"),
        "expected claim success message in achievements body:\n{body}"
    );

    fixture.assert_claimed(&pool, 17, 3).await;
    fixture.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_claim_get_query_does_not_mutate() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-achievement-get");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = AchievementScenario::attach_to_user(&pool, &prefix, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert(
        "achievementId".to_string(),
        fixture.achievement_id.to_string(),
    );

    let response = route(
        &get_request_query("/achievements", query, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "achievements GET should still render");
    assert!(
        !body.contains("Achievement claimed"),
        "GET query must not claim an achievement:\n{body}"
    );

    let steps_claimed: i32 = sqlx::query_scalar(
        "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
    )
    .bind(fixture.user_id)
    .bind(fixture.achievement_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load claimed steps");
    assert_eq!(steps_claimed, 0);

    fixture.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_overview_renders_other_players_tracks() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-ach-overview");
    let viewer = format!("{prefix}-viewer");
    let target = format!("{prefix}-target");
    let password = "test-password-1".to_string();
    create_user_via_engine(
        &viewer,
        &format!("{prefix}-viewer@example.invalid"),
        &password,
    );
    let target_user_id = create_user_via_engine(
        &target,
        &format!("{prefix}-target@example.invalid"),
        &password,
    );
    let fixture = AchievementScenario::attach_to_user(&pool, &prefix, target_user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &viewer, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("user".to_string(), target.clone());
    let response = route(
        &get_request_query("/achievements", query, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "{body}");
    assert!(
        body.contains("achievements-page-overview"),
        "expected overview page:\n{body}"
    );
    assert!(
        body.contains(&target) && body.contains("achievements"),
        "expected other player's overview title for {target}:\n{body}"
    );
    assert!(
        !body.contains("achievement-claim-badge"),
        "overview must not offer Claim:\n{body}"
    );

    fixture.cleanup(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_overview_unknown_user_shows_not_found() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-ach-missing");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());
    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert("user".to_string(), format!("{prefix}-definitely-missing"));
    let response = route(
        &get_request_query("/achievements", query, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "{body}");
    assert!(
        body.contains("Player not found."),
        "expected player not found:\n{body}"
    );
    assert!(
        !body.contains("achievement-claim-badge"),
        "not-found overview must not offer Claim:\n{body}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn achievements_claim_rejected_shows_unable_banner() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-achievement-reject");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = AchievementScenario::attach_to_user(&pool, &prefix, user_id).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("achievementId".to_string(), "999999999".to_string());

    let response = route(&post_request("/achievements", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "{body}");
    assert!(
        body.contains("Unable to claim achievement:"),
        "expected claim rejection banner:\n{body}"
    );
    assert!(
        !body.contains("Achievement claimed"),
        "bogus achievement must not claim:\n{body}"
    );

    let steps_claimed: i32 = sqlx::query_scalar(
        "SELECT stepsClaimed FROM UserAchievement WHERE userId = ? AND achievementId = ?",
    )
    .bind(fixture.user_id)
    .bind(fixture.achievement_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load claimed steps");
    assert_eq!(steps_claimed, 0);

    fixture.cleanup(&pool, true).await;
}
