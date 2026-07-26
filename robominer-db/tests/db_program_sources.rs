use robominer_db::{ProgramSourceWriteRejection, delete_program_source_for_user};
use robominer_test_support::{ProgramSourceFixture, insert_row_id};
use serial_test::serial;
use sqlx::Row;

#[tokio::test]
#[serial]
async fn delete_program_source_for_user_rejects_linked_robot() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;
    let robot_id = fixture
        .insert_linked_robot(&pool, "linked", 128, "move(1);")
        .await;
    fixture.robot_ids.borrow_mut().push(robot_id);

    let rejection =
        delete_program_source_for_user(&pool, fixture.user_id, fixture.program_source_id)
            .await
            .expect("delete should not fail at sql layer")
            .expect_err("linked robot should block delete");

    assert_eq!(rejection, ProgramSourceWriteRejection::SourceInUse);

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ProgramSource WHERE id = ?")
        .bind(fixture.program_source_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count program source");
    assert_eq!(remaining, 1);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn delete_program_source_for_user_removes_unused_source() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;
    let unused_source_id = insert_row_id(
        &pool,
        sqlx::query(
            "INSERT INTO ProgramSource \
             (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
             VALUES (?, 'unused source', 'mine();', true, 1, '')",
        )
        .bind(fixture.user_id),
    )
    .await;

    delete_program_source_for_user(&pool, fixture.user_id, unused_source_id)
        .await
        .expect("delete should not fail at sql layer")
        .expect("unused source should delete");

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ProgramSource WHERE id = ?")
        .bind(unused_source_id)
        .fetch_one(&pool)
        .await
        .expect("failed to count deleted program source");
    assert_eq!(remaining, 0);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn delete_program_source_for_user_rejects_foreign_source() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;

    let rejection =
        delete_program_source_for_user(&pool, fixture.other_user_id, fixture.program_source_id)
            .await
            .expect("delete should not fail at sql layer")
            .expect_err("other user should not delete foreign source");

    assert_eq!(rejection, ProgramSourceWriteRejection::UnknownProgramSource);

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn update_program_source_clears_verified() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;

    let verified_before: bool =
        sqlx::query_scalar("SELECT verified FROM ProgramSource WHERE id = ?")
            .bind(fixture.program_source_id)
            .fetch_one(&pool)
            .await
            .expect("load verified");
    assert!(verified_before);

    robominer_db::update_program_source(
        &pool,
        robominer_db::ProgramSourceWriteRequest {
            user_id: fixture.user_id,
            program_source_id: fixture.program_source_id,
            source_name: "updated source".to_string(),
            source_code: "dump();".to_string(),
        },
    )
    .await
    .expect("sql")
    .expect("update own source");

    let row =
        sqlx::query("SELECT sourceName, sourceCode, verified FROM ProgramSource WHERE id = ?")
            .bind(fixture.program_source_id)
            .fetch_one(&pool)
            .await
            .expect("load after update");
    let name: String = row.try_get("sourceName").unwrap();
    let code: String = row.try_get("sourceCode").unwrap();
    let verified: bool = row.try_get("verified").unwrap();
    assert_eq!(name, "updated source");
    assert_eq!(code, "dump();");
    assert!(!verified);

    let rejection = robominer_db::update_program_source(
        &pool,
        robominer_db::ProgramSourceWriteRequest {
            user_id: fixture.other_user_id,
            program_source_id: fixture.program_source_id,
            source_name: "hijack".to_string(),
            source_code: "mine();".to_string(),
        },
    )
    .await
    .expect("sql")
    .expect_err("foreign user cannot update");
    assert_eq!(rejection, ProgramSourceWriteRejection::UnknownProgramSource);

    let code_after: String =
        sqlx::query_scalar("SELECT sourceCode FROM ProgramSource WHERE id = ?")
            .bind(fixture.program_source_id)
            .fetch_one(&pool)
            .await
            .expect("code unchanged");
    assert_eq!(code_after, "dump();");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn set_valid_and_invalid_program_source_flags() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;

    robominer_db::set_invalid_program_source(&pool, fixture.program_source_id, "bad compile")
        .await
        .expect("set_invalid");
    let invalid = sqlx::query(
        "SELECT verified, compiledSize, errorDescription FROM ProgramSource WHERE id = ?",
    )
    .bind(fixture.program_source_id)
    .fetch_one(&pool)
    .await
    .expect("load invalid");
    assert!(!invalid.try_get::<bool, _>("verified").unwrap());
    assert_eq!(invalid.try_get::<i32, _>("compiledSize").unwrap(), -1);
    assert_eq!(
        invalid.try_get::<String, _>("errorDescription").unwrap(),
        "bad compile"
    );

    robominer_db::set_valid_program_source(&pool, fixture.program_source_id, 42)
        .await
        .expect("set_valid");
    let valid = sqlx::query(
        "SELECT verified, compiledSize, errorDescription FROM ProgramSource WHERE id = ?",
    )
    .bind(fixture.program_source_id)
    .fetch_one(&pool)
    .await
    .expect("load valid");
    assert!(valid.try_get::<bool, _>("verified").unwrap());
    assert_eq!(valid.try_get::<i32, _>("compiledSize").unwrap(), 42);
    assert_eq!(valid.try_get::<String, _>("errorDescription").unwrap(), "");

    fixture.cleanup(&pool).await;
}

#[tokio::test]
#[serial]
async fn get_program_source_and_verification_cover_states() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping robominer-db program source test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;

    let source = robominer_db::get_program_source(&pool, fixture.program_source_id)
        .await
        .expect("get source")
        .expect("source exists");
    assert!(!source.is_empty());

    assert!(
        robominer_db::get_program_source(&pool, -1)
            .await
            .expect("missing source query")
            .is_none()
    );

    let verified = robominer_db::get_program_source_verification(&pool, fixture.program_source_id)
        .await
        .expect("get verification")
        .expect("verification exists");
    assert!(verified.verified);
    assert!(verified.compiled_size >= 0);
    assert!(verified.error_description.is_empty());

    robominer_db::set_invalid_program_source(&pool, fixture.program_source_id, "syntax boom")
        .await
        .expect("set_invalid");
    let invalid = robominer_db::get_program_source_verification(&pool, fixture.program_source_id)
        .await
        .expect("get invalid verification")
        .expect("verification exists");
    assert!(!invalid.verified);
    assert_eq!(invalid.compiled_size, -1);
    assert_eq!(invalid.error_description, "syntax boom");

    assert!(
        robominer_db::get_program_source_verification(&pool, -1)
            .await
            .expect("missing verification query")
            .is_none()
    );

    fixture.cleanup(&pool).await;
}
