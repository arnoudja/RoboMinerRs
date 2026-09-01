#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::collections::HashMap;

use robominer_web::test_support::{
    MAX_ATTEMPTS_PER_LOGIN, MAX_MUTATIONS_PER_USER_ACTION, lock_auth_rate_limiter_for_tests,
    reset_auth_rate_limiter_for_tests, reset_mutation_rate_limiter_for_tests, route,
};
use serial_test::serial;
use support::{
    anonymous_login_csrf, apply_set_cookies, cookie_header, create_user_via_engine,
    ensure_session_configured, get_request, login_with_credentials, post_request, response_body,
    server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn post_logoff_through_route_clears_session_and_blocks_protected_pages() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();
    reset_mutation_rate_limiter_for_tests();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-logoff");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    assert_eq!(login_response.status, 302);
    let session_cookie = cookie_header(&login_response);

    let queue_before = route(&get_request("/miningQueue", Some(&session_cookie)), &config).await;
    assert_eq!(queue_before.status, 200);

    let logoff_response = route(
        &post_request("/logoff", HashMap::new(), Some(&session_cookie)),
        &config,
    )
    .await;
    assert_eq!(logoff_response.status, 200);
    assert!(
        logoff_response
            .headers
            .iter()
            .any(|(name, value)| *name == "Set-Cookie"
                && value.starts_with("robominer_session=; Max-Age=0;")),
        "POST /logoff should clear the session cookie"
    );

    let cleared_cookie = apply_set_cookies(&session_cookie, &logoff_response);
    assert!(
        !cleared_cookie.contains("robominer_session="),
        "browser should drop the session cookie after logoff"
    );

    let queue_after = route(&get_request("/miningQueue", Some(&cleared_cookie)), &config).await;
    assert_eq!(
        queue_after.status, 302,
        "protected page should require login after logoff"
    );
    assert!(
        queue_after
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value.starts_with("login")),
        "cleared session should redirect to login"
    );

    let session_version: i32 = sqlx::query_scalar("SELECT sessionVersion FROM User WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("session version");
    assert_eq!(
        session_version, 1,
        "logoff should bump sessionVersion so stolen cookies are invalid"
    );

    let stale_queue = route(&get_request("/miningQueue", Some(&session_cookie)), &config).await;
    assert_eq!(
        stale_queue.status, 302,
        "pre-logoff session cookie must be rejected after sessionVersion bump"
    );

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn authenticated_shop_post_returns_429_after_mutation_limit() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();
    reset_mutation_rate_limiter_for_tests();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-rate-limit");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let mut cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("selectedRobotPartTypeId".to_string(), "1".to_string());
    form.insert("selectedTierId".to_string(), "1".to_string());
    form.insert("selectedRobotPartId".to_string(), "1".to_string());

    for index in 0..=MAX_MUTATIONS_PER_USER_ACTION {
        form.insert("buyRobotPartId".to_string(), "999999".to_string());
        let response = route(&post_request("/shop", form.clone(), Some(&cookie)), &config).await;
        cookie = apply_set_cookies(&cookie, &response);

        if index < MAX_MUTATIONS_PER_USER_ACTION {
            assert_ne!(
                response.status, 429,
                "request {index} should not be rate limited yet"
            );
            continue;
        }

        assert_eq!(
            response.status, 429,
            "request {index} should be rate limited"
        );
        let body = response_body(&response);
        assert!(
            body.contains("Too many requests"),
            "429 body should explain the limit:\n{body}"
        );
    }

    // Mutation families are independent: shop exhaustion must not block edit-code.
    let mut edit_form = HashMap::new();
    edit_form.insert("requestType".to_string(), "update".to_string());
    edit_form.insert("programSourceId".to_string(), "-1".to_string());
    edit_form.insert("nextProgramSourceId".to_string(), "-1".to_string());
    edit_form.insert("sourceName".to_string(), format!("{prefix}-program"));
    edit_form.insert("sourceCode".to_string(), "move(1);".to_string());
    let edit_response = route(
        &post_request("/editCode", edit_form, Some(&cookie)),
        &config,
    )
    .await;
    assert_ne!(
        edit_response.status, 429,
        "edit-code family should still succeed after shop rate limit"
    );
    assert_eq!(
        edit_response.status, 200,
        "edit-code POST should render after shop family is exhausted"
    );
    cookie = apply_set_cookies(&cookie, &edit_response);

    // Mining-queue family is also independent of shop exhaustion.
    let mut queue_form = HashMap::new();
    queue_form.insert("submitType".to_string(), "clear".to_string());
    let queue_response = route(
        &post_request("/miningQueue", queue_form, Some(&cookie)),
        &config,
    )
    .await;
    assert_ne!(
        queue_response.status, 429,
        "mining-queue family should still succeed after shop rate limit"
    );
    assert_eq!(
        queue_response.status, 200,
        "mining-queue POST should render after shop family is exhausted"
    );

    let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn stale_session_version_is_rejected_against_database() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();
    reset_mutation_rate_limiter_for_tests();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-session-version");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let stale_cookie = cookie_header(&login_response);

    sqlx::query("UPDATE User SET sessionVersion = sessionVersion + 1 WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("bump session version");

    let response = route(&get_request("/shop", Some(&stale_cookie)), &config).await;
    assert_eq!(response.status, 302, "stale session should not load shop");
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| *name == "Location" && value.starts_with("login")),
        "stale session should redirect to login"
    );
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| *name == "Set-Cookie"
                && value.starts_with("robominer_session=; Max-Age=0;")),
        "stale session should clear the session cookie"
    );

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn login_post_returns_429_after_auth_rate_limit() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();
    let _guard = lock_auth_rate_limiter_for_tests();
    reset_auth_rate_limiter_for_tests();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-login-rate");
    let username = format!("{prefix}-user");
    let user_id = create_user_via_engine(
        &username,
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    );
    let config = server_config(pool.clone());

    let (csrf_cookie, token) = anonymous_login_csrf(&config).await;
    let mut form = HashMap::new();
    form.insert("loginName".to_string(), username.clone());
    form.insert("password".to_string(), "wrong-password".to_string());

    for index in 0..=MAX_ATTEMPTS_PER_LOGIN {
        form.insert("csrfToken".to_string(), token.clone());
        let response = route(
            &post_request("/login", form.clone(), Some(&csrf_cookie)),
            &config,
        )
        .await;

        if index < MAX_ATTEMPTS_PER_LOGIN {
            assert_ne!(
                response.status, 429,
                "login attempt {index} should not be rate limited yet"
            );
            continue;
        }

        assert_eq!(
            response.status, 429,
            "login attempt {index} should be rate limited"
        );
        let body = response_body(&response);
        assert!(
            body.contains("Too many login attempts"),
            "429 body should explain the limit:\n{body}"
        );
    }

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn account_update_post_returns_429_after_auth_rate_limit() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    ensure_session_configured();
    let _guard = lock_auth_rate_limiter_for_tests();
    reset_auth_rate_limiter_for_tests();

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-account-rate");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    assert_eq!(login_response.status, 302);
    let session_cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("username".to_string(), username.clone());
    form.insert("email".to_string(), format!("{prefix}@example.invalid"));
    form.insert("currentpassword".to_string(), "wrong-password".to_string());

    for index in 0..=MAX_ATTEMPTS_PER_LOGIN {
        let response = route(
            &post_request("/account", form.clone(), Some(&session_cookie)),
            &config,
        )
        .await;

        if index < MAX_ATTEMPTS_PER_LOGIN {
            assert_ne!(
                response.status, 429,
                "account update attempt {index} should not be rate limited yet"
            );
            continue;
        }

        assert_eq!(
            response.status, 429,
            "account update attempt {index} should be rate limited"
        );
        let body = response_body(&response);
        assert!(
            body.contains("Too many account password checks"),
            "429 body should explain the limit:\n{body}"
        );
    }

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
