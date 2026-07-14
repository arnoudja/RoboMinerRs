use robominer_db::{ProgramSourceWriteRejection, delete_program_source_for_user};
use robominer_test_support::{ProgramSourceFixture, insert_row_id};
use serial_test::serial;

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

    let rejection = delete_program_source_for_user(
        &pool,
        fixture.user_id,
        fixture.program_source_id,
    )
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

    let rejection = delete_program_source_for_user(
        &pool,
        fixture.other_user_id,
        fixture.program_source_id,
    )
    .await
    .expect("delete should not fail at sql layer")
    .expect_err("other user should not delete foreign source");

    assert_eq!(
        rejection,
        ProgramSourceWriteRejection::UnknownProgramSource
    );

    fixture.cleanup(&pool).await;
}
