#![allow(clippy::unwrap_used, clippy::expect_used)]
use robominer_domain::create_program_source;
use robominer_test_support::ProgramSourceFixture;
use serial_test::serial;

/// Densest program whose compiled size is 24_576 (Enhanced Etaxy).
/// A sequence of n>1 statements costs 1 + n, and each `mine();` costs 1.
fn largest_memory_module_program_source() -> String {
    const STATEMENT_COUNT: usize = 24_575;
    let mut source = String::with_capacity(STATEMENT_COUNT * 7);
    for _ in 0..STATEMENT_COUNT {
        source.push_str("mine();\n");
    }
    source
}

#[tokio::test]
#[serial]
async fn create_program_source_stores_compiled_size_of_largest_memory_module() {
    let Some(database_url) = robominer_test_support::require_test_db() else {
        return;
    };

    let pool = robominer_db::connect(&database_url)
        .await
        .expect("failed to connect to test database");
    let fixture = ProgramSourceFixture::create(&pool).await;
    let source = largest_memory_module_program_source();
    assert!(
        source.len() > 16_384,
        "fixture must exceed the old 16 KiB source cap"
    );

    let created = create_program_source(
        &pool,
        robominer_db::CreateProgramSourceRequest {
            user_id: fixture.user_id,
            source_name: "max-memory".to_string(),
            source_code: source.clone(),
        },
    )
    .await
    .expect("create should not fail at sql layer")
    .into_result()
    .expect("compiled size 24576 should be accepted");

    let row =
        sqlx::query("SELECT sourceCode, compiledSize, verified FROM ProgramSource WHERE id = ?")
            .bind(created.program_source_id)
            .fetch_one(&pool)
            .await
            .expect("load saved program");
    let stored: String = sqlx::Row::try_get(&row, "sourceCode").unwrap();
    let compiled_size: i32 = sqlx::Row::try_get(&row, "compiledSize").unwrap();
    let verified: bool = sqlx::Row::try_get(&row, "verified").unwrap();
    assert_eq!(stored, source);
    assert!(verified);
    assert_eq!(compiled_size, 24_576);

    fixture.cleanup(&pool).await;
}
