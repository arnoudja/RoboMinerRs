mod support;

use std::collections::HashMap;

use robominer_test_support::insert_row_id;
use robominer_web::test_support::route;
use serial_test::serial;
use support::{
    apply_set_cookies, cookie_header, create_user_via_engine, ensure_session_configured,
    login_with_credentials, post_request, response_body, server_config, unique_prefix,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn edit_code_create_post_inserts_program_source() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping edit code web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-edit-code");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());

    let login_response = login_with_credentials(&config, &username, &password).await;
    let cookie = cookie_header(&login_response);

    let mut form = HashMap::new();
    form.insert("requestType".to_string(), "update".to_string());
    form.insert("programSourceId".to_string(), "0".to_string());
    form.insert("sourceName".to_string(), format!("{prefix}-program"));
    form.insert("sourceCode".to_string(), "move(1);".to_string());

    let response = route(&post_request("/editCode", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "edit code page should render");
    assert!(
        body.contains("Program created."),
        "expected create success message in edit code body:\n{body}"
    );

    let program_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ProgramSource WHERE userId = ? AND sourceName = ?",
    )
    .bind(user_id)
    .bind(format!("{prefix}-program"))
    .fetch_one(&pool)
    .await
    .expect("failed to count program sources");
    assert_eq!(program_count, 1);

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
async fn edit_code_apply_post_updates_linked_robots() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping edit code apply web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-edit-apply");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());
    let cookie = cookie_header(&login_with_credentials(&config, &username, &password).await);

    let program_source_id: i64 = sqlx::query_scalar(
        "SELECT programSourceId FROM Robot WHERE userId = ? ORDER BY id LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("failed to load linked program source id");

    let mut form = HashMap::new();
    form.insert("requestType".to_string(), "applyRobots".to_string());
    form.insert("programSourceId".to_string(), program_source_id.to_string());
    form.insert(
        "nextProgramSourceId".to_string(),
        program_source_id.to_string(),
    );

    let response = route(&post_request("/editCode", form, Some(&cookie)), &config).await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "edit code page should render");
    assert!(
        body.contains("Updated 1 robot(s)."),
        "expected apply success message in edit code body:\n{body}"
    );

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
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
async fn edit_code_delete_post_removes_unlinked_program_source() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping edit code delete web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-edit-delete");
    let username = format!("{prefix}-user");
    let password = "test-password-1".to_string();
    let user_id =
        create_user_via_engine(&username, &format!("{prefix}@example.invalid"), &password);
    let config = server_config(pool.clone());
    let cookie = cookie_header(&login_with_credentials(&config, &username, &password).await);
    let program_name = format!("{prefix}-delete-me");

    let mut create_form = HashMap::new();
    create_form.insert("requestType".to_string(), "update".to_string());
    create_form.insert("programSourceId".to_string(), "0".to_string());
    create_form.insert("sourceName".to_string(), program_name.clone());
    create_form.insert("sourceCode".to_string(), "move(1);".to_string());

    let create_response = route(
        &post_request("/editCode", create_form, Some(&cookie)),
        &config,
    )
    .await;
    assert_eq!(
        create_response.status, 200,
        "edit code create should render before delete"
    );
    let cookie = apply_set_cookies(&cookie, &create_response);

    let program_source_id: i64 =
        sqlx::query_scalar("SELECT id FROM ProgramSource WHERE userId = ? AND sourceName = ?")
            .bind(user_id)
            .bind(&program_name)
            .fetch_one(&pool)
            .await
            .expect("failed to load created program source id");

    let mut delete_form = HashMap::new();
    delete_form.insert("requestType".to_string(), "erase".to_string());
    delete_form.insert("programSourceId".to_string(), program_source_id.to_string());

    let response = route(
        &post_request("/editCode", delete_form, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "edit code page should render");
    assert!(
        body.contains("Program deleted."),
        "expected delete success message in edit code body:\n{body}"
    );

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ProgramSource WHERE id = ?")
        .bind(program_source_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count deleted program source");
    assert_eq!(remaining, 0);

    let _ = sqlx::query("DELETE FROM Robot WHERE userId = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM ProgramSource WHERE userId = ?")
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
async fn edit_code_delete_post_rejects_foreign_program_source() {
    if std::env::var("ROBOMINER_DATABASE_URL").is_err() {
        eprintln!("skipping edit code IDOR web test: ROBOMINER_DATABASE_URL is not set");
        return;
    }

    ensure_session_configured();

    let pool =
        robominer_db::connect(&std::env::var("ROBOMINER_DATABASE_URL").expect("database url"))
            .await
            .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-web-edit-idor");
    let owner_username = format!("{prefix}-owner");
    let attacker_username = format!("{prefix}-attacker");
    let password = "test-password-1".to_string();
    let owner_id = create_user_via_engine(
        &owner_username,
        &format!("{prefix}-owner@example.invalid"),
        &password,
    );
    let attacker_id = create_user_via_engine(
        &attacker_username,
        &format!("{prefix}-attacker@example.invalid"),
        &password,
    );

    let program_name = format!("{prefix}-secret");
    let program_source_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO ProgramSource \
             (userId, sourceName, sourceCode, compiledSize, errorDescription, verified) \
             VALUES (?, ?, 'move(1);', 1, '', 1)",
        )
        .bind(owner_id)
        .bind(&program_name),
    )
    .await;

    let config = server_config(pool.clone());
    let cookie =
        cookie_header(&login_with_credentials(&config, &attacker_username, &password).await);

    let mut delete_form = HashMap::new();
    delete_form.insert("requestType".to_string(), "erase".to_string());
    delete_form.insert("programSourceId".to_string(), program_source_id.to_string());

    let response = route(
        &post_request("/editCode", delete_form, Some(&cookie)),
        &config,
    )
    .await;
    let body = response_body(&response);

    assert_eq!(response.status, 200, "edit code page should render");
    assert!(
        body.contains("Unknown program source."),
        "expected IDOR rejection for foreign program source:\n{body}"
    );

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ProgramSource WHERE id = ?")
        .bind(program_source_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count owner program source");
    assert_eq!(
        remaining, 1,
        "foreign erase must not delete the owner program"
    );

    for user_id in [owner_id, attacker_id] {
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
}
