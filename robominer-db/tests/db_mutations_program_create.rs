#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_db::{CreateProgramSourceRequest, create_program_source};
use robominer_test_support::{insert_user_with_credentials, unique_prefix};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_program_source_rejects_empty_source_name() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-program");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;

    let rejection = create_program_source(
        &pool,
        CreateProgramSourceRequest {
            user_id,
            source_name: String::new(),
            source_code: "mine();".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect_err("empty source name should reject");

    assert_eq!(
        rejection,
        robominer_db::ProgramSourceWriteRejection::EmptySourceName
    );

    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn create_program_source_inserts_verifiable_row() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let prefix = unique_prefix("rust-db-program");
    let user_id = insert_user_with_credentials(
        &pool,
        &format!("{prefix}-user"),
        &format!("{prefix}@example.invalid"),
        "test-password-1",
    )
    .await;

    let created = create_program_source(
        &pool,
        CreateProgramSourceRequest {
            user_id,
            source_name: format!("{prefix}-source"),
            source_code: "move(1);".to_string(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .expect("create should succeed");

    let (source_name, verified): (String, bool) =
        sqlx::query_as("SELECT sourceName, verified FROM ProgramSource WHERE id = ?")
            .bind(created.program_source_id)
            .fetch_one(&pool)
            .await
            .expect("failed to load created program source");
    assert_eq!(source_name, format!("{prefix}-source"));
    assert!(!verified);

    let _ = sqlx::query("DELETE FROM ProgramSource WHERE id = ?")
        .bind(created.program_source_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM User WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await;
}
