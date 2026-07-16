mod support;

use std::collections::HashMap;

use robominer_test_support::ShopFixture;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    cookie_header, create_user_via_engine, ensure_session_configured, get_request,
    get_request_query, login_with_credentials, post_request, post_request_without_csrf,
    response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn shop_buy_post_deducts_ore_and_adds_owned_part() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping shop buy web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-shop");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = ShopFixture::attach_to_user(&pool, &prefix, user_id, 25, 10, 0).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    assert_eq!(login_response.status, 302);
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "buyRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );
    form.insert(
        "selectedRobotPartTypeId".to_string(),
        fixture.robot_part_type_id.to_string(),
    );
    form.insert("selectedTierId".to_string(), fixture.ore_id.to_string());
    form.insert(
        "selectedRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );

    let response = route(&post_request("/shop", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "shop buy should render");
    assert!(
        body.contains("Robot part bought"),
        "expected buy success message in shop body:\n{body}"
    );

    fixture.assert_ore_amount(&pool, 15).await;
    fixture.assert_robot_part_total_owned(&pool, Some(1)).await;
    fixture.cleanup_attached(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn shop_sell_post_refunds_ore_and_clears_owned_part() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping shop sell web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-shop");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = ShopFixture::attach_to_user(&pool, &prefix, user_id, 0, 10, 1).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "sellRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );
    form.insert(
        "selectedRobotPartTypeId".to_string(),
        fixture.robot_part_type_id.to_string(),
    );
    form.insert("selectedTierId".to_string(), fixture.ore_id.to_string());
    form.insert(
        "selectedRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );

    let response = route(&post_request("/shop", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "shop sell should render");
    assert!(
        body.contains("Robot part sold"),
        "expected sell success message in shop body:\n{body}"
    );

    fixture.assert_ore_amount(&pool, 5).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup_attached(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn shop_buy_post_shows_insufficient_funds_message() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping shop insufficient funds web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-shop-funds");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = ShopFixture::attach_to_user(&pool, &prefix, user_id, 5, 10, 0).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "buyRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );
    form.insert(
        "selectedRobotPartTypeId".to_string(),
        fixture.robot_part_type_id.to_string(),
    );
    form.insert("selectedTierId".to_string(), fixture.ore_id.to_string());
    form.insert(
        "selectedRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );

    let response = route(&post_request("/shop", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "shop buy rejection should render");
    assert!(
        body.contains("insufficient funds"),
        "expected insufficient funds message in shop body:\n{body}"
    );

    fixture.assert_ore_amount(&pool, 5).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup_attached(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn shop_buy_get_query_does_not_mutate() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping shop GET buy web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-shop-get");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = ShopFixture::attach_to_user(&pool, &prefix, user_id, 25, 10, 0).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut query = HashMap::new();
    query.insert(
        "buyRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );
    query.insert(
        "selectedRobotPartTypeId".to_string(),
        fixture.robot_part_type_id.to_string(),
    );
    query.insert("selectedTierId".to_string(), fixture.ore_id.to_string());
    query.insert(
        "selectedRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );

    let response = route(&get_request_query("/shop", query, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "shop GET should still render");
    assert!(
        !body.contains("Robot part bought"),
        "GET query must not buy a part:\n{body}"
    );
    assert!(
        body.contains(r#"name="csrf-token""#) && body.contains(r#"name="csrfToken""#),
        "authenticated shop page should embed CSRF tokens:\n{body}"
    );

    fixture.assert_ore_amount(&pool, 25).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup_attached(&pool, true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn shop_buy_post_without_csrf_is_rejected() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping shop CSRF web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-shop-csrf");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let fixture = ShopFixture::attach_to_user(&pool, &prefix, user_id, 25, 10, 0).await;
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert(
        "buyRobotPartId".to_string(),
        fixture.robot_part_id.to_string(),
    );

    let missing = route(
        &post_request_without_csrf("/shop", form.clone(), Some(&cookie)),
        &config,
    )
    .await;
    assert_eq!(missing.status, 403);
    assert!(
        response_body(&missing).contains("CSRF"),
        "expected CSRF rejection message"
    );

    form.insert("csrfToken".to_string(), "not-a-valid-token".to_string());
    let forged = route(&post_request("/shop", form, Some(&cookie)), &config).await;
    assert_eq!(forged.status, 403);

    let page = route(&get_request("/shop", Some(&cookie)), &config).await;
    assert_eq!(page.status, 200);

    fixture.assert_ore_amount(&pool, 25).await;
    fixture.assert_robot_part_total_owned(&pool, None).await;
    fixture.cleanup_attached(&pool, true).await;
}
