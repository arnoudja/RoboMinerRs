use robominer_db::{
    EMBEDDED_MIGRATIONS, migration_status, run_embedded_migrations, run_migrations,
};
use serial_test::serial;

fn database_url() -> Option<String> {
    std::env::var("ROBOMINER_DATABASE_URL").ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn migrate_on_current_schema_baselines_then_is_idempotent() {
    let Some(database_url) = database_url() else {
        eprintln!("skipping migrate test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url).await.expect("connect");

    // Clear tracking so the runner can re-baseline a current schema.
    let _ = sqlx::query("DELETE FROM SchemaMigration")
        .execute(&pool)
        .await;

    let first = run_embedded_migrations(&pool).await.expect("first migrate");
    assert!(
        !first.baselined.is_empty() || !first.applied.is_empty(),
        "expected migrations to baseline or apply: {first:?}"
    );
    assert!(
        first.already_applied.is_empty(),
        "fresh tracking should not report already-applied: {first:?}"
    );

    let second = run_embedded_migrations(&pool)
        .await
        .expect("second migrate");
    assert!(
        second.applied.is_empty(),
        "second migrate should not re-apply"
    );
    assert!(
        second.baselined.is_empty(),
        "second migrate should not re-baseline"
    );
    assert_eq!(second.already_applied.len(), EMBEDDED_MIGRATIONS.len());

    let status = migration_status(&pool, EMBEDDED_MIGRATIONS)
        .await
        .expect("status");
    assert!(status.iter().all(|(_, applied)| *applied));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn migrate_applies_pending_version_when_tracking_is_partial() {
    let Some(database_url) = database_url() else {
        eprintln!("skipping migrate partial test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let pool = robominer_db::connect(&database_url).await.expect("connect");

    let _ = sqlx::query("DELETE FROM SchemaMigration")
        .execute(&pool)
        .await;
    run_embedded_migrations(&pool)
        .await
        .expect("baseline current schema");

    // Simulate a brand-new pending migration that is a no-op on current schema.
    let pending = [(
        "999_migrate_runner_self_test",
        "-- self-test no-op migration\nSELECT 1;",
    )];
    let report = run_migrations(&pool, &pending)
        .await
        .expect("apply self-test migration");
    assert_eq!(
        report.applied,
        vec!["999_migrate_runner_self_test".to_string()]
    );

    let again = run_migrations(&pool, &pending)
        .await
        .expect("re-run self-test migration");
    assert_eq!(
        again.already_applied,
        vec!["999_migrate_runner_self_test".to_string()]
    );

    let _ = sqlx::query("DELETE FROM SchemaMigration WHERE version = ?")
        .bind("999_migrate_runner_self_test")
        .execute(&pool)
        .await;
}
