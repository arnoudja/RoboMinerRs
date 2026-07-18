mod support;
use serial_test::serial;

use support::*;

#[tokio::test]
#[serial]
async fn migrate_status_lists_embedded_migrations() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url.clone(),
        "migrate-status".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected migrate-status to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("001_rename_scan_speed_to_scan_time"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(
        stdout.contains('\t'),
        "expected version/status columns in stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}

#[tokio::test]
#[serial]
async fn migrate_is_idempotent_on_current_schema() {
    let Ok(database_url) = std::env::var("ROBOMINER_DATABASE_URL") else {
        eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
        return;
    };

    let output = run_engine(&[
        "--database-url".to_string(),
        database_url,
        "migrate".to_string(),
    ]);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "expected migrate to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("already-applied") || stdout.contains("no-migrations") || stdout.contains("baselined") || stdout.contains("applied"),
        "unexpected stdout:\n{stdout}"
    );
    assert!(stderr.is_empty(), "unexpected stderr:\n{stderr}");
}
