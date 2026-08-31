use std::path::{Path, PathBuf};

use super::errors::MigrateError;

pub fn default_migrations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../resources/database/migrations")
}

pub fn load_migrations_from_dir(
    migrations_dir: &Path,
) -> Result<Vec<(String, String)>, MigrateError> {
    let mut entries = std::fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "sql")
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    let mut migrations = Vec::new();
    for entry in entries {
        let path = entry.path();
        let Some(version) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !version.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
            return Err(MigrateError::InvalidMigration(format!(
                "migration file {} must start with a numeric version prefix",
                path.display()
            )));
        }
        let sql = std::fs::read_to_string(&path)?;
        migrations.push((version.to_string(), sql));
    }
    Ok(migrations)
}

pub fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(trimmed);
        if trimmed.ends_with(';') {
            let statement = current.trim_end_matches(';').trim().to_string();
            if !statement.is_empty() {
                statements.push(statement);
            }
            current.clear();
        }
    }

    let trailing = current.trim().trim_end_matches(';').trim();
    if !trailing.is_empty() {
        statements.push(trailing.to_string());
    }

    statements
}

#[cfg(test)]
mod tests {
    use super::{load_migrations_from_dir, split_sql_statements};

    #[test]
    fn split_sql_statements_skips_comments_and_blank_lines() {
        let sql = "-- heading\n\nALTER TABLE t ADD c INT;\nUPDATE t SET c = 1;\n";
        assert_eq!(
            split_sql_statements(sql),
            vec![
                "ALTER TABLE t ADD c INT".to_string(),
                "UPDATE t SET c = 1".to_string()
            ]
        );
    }

    #[test]
    fn load_migrations_rejects_non_numeric_prefix() {
        let dir =
            std::env::temp_dir().join(format!("robominer-migrate-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp migrations dir");
        std::fs::write(dir.join("not_a_version.sql"), "SELECT 1;").expect("write bad migration");
        let error = load_migrations_from_dir(&dir).expect_err("non-numeric prefix");
        let _ = std::fs::remove_dir_all(&dir);
        let message = error.to_string();
        assert!(
            message.contains("must start with a numeric version prefix"),
            "{message}"
        );
    }
}
