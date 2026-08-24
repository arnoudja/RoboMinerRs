//! Helpers for integration tests that require MySQL.

/// Returns the test database URL when `ROBOMINER_DATABASE_URL` is set.
///
/// When the variable is missing and `CI=true`, panics so CI misconfiguration
/// cannot appear as a green run with silently skipped tests. Otherwise prints a
/// skip message and returns `None`.
pub fn require_test_db() -> Option<String> {
    match std::env::var("ROBOMINER_DATABASE_URL") {
        Ok(url) if !url.is_empty() => Some(url),
        _ => {
            if std::env::var("CI").ok().as_deref() == Some("true") {
                panic!("ROBOMINER_DATABASE_URL must be set when CI=true");
            }
            eprintln!("skipping DB integration test: ROBOMINER_DATABASE_URL is not set");
            None
        }
    }
}
