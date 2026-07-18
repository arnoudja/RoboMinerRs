use std::path::{Path, PathBuf};

use sqlx::{MySqlPool, Row};

/// Embedded schema migrations, ordered by version prefix.
pub const EMBEDDED_MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_rename_scan_speed_to_scan_time",
        include_str!("../../resources/database/migrations/001_rename_scan_speed_to_scan_time.sql"),
    ),
    (
        "002_mining_queue_executed_source_code",
        include_str!(
            "../../resources/database/migrations/002_mining_queue_executed_source_code.sql"
        ),
    ),
    (
        "003_user_session_version",
        include_str!("../../resources/database/migrations/003_user_session_version.sql"),
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub applied: Vec<String>,
    pub baselined: Vec<String>,
    pub already_applied: Vec<String>,
}

#[derive(Debug)]
pub enum MigrateError {
    Database(sqlx::Error),
    InvalidMigration(String),
    Io(std::io::Error),
}

impl std::fmt::Display for MigrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
            Self::InvalidMigration(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MigrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidMigration(_) => None,
        }
    }
}

impl From<sqlx::Error> for MigrateError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<std::io::Error> for MigrateError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub async fn run_embedded_migrations(pool: &MySqlPool) -> Result<MigrationReport, MigrateError> {
    run_migrations(pool, EMBEDDED_MIGRATIONS).await
}

pub async fn run_migrations_from_dir(
    pool: &MySqlPool,
    migrations_dir: &Path,
) -> Result<MigrationReport, MigrateError> {
    let migrations = load_migrations_from_dir(migrations_dir)?;
    let borrowed: Vec<(&str, &str)> = migrations
        .iter()
        .map(|(version, sql)| (version.as_str(), sql.as_str()))
        .collect();
    run_migrations(pool, &borrowed).await
}

pub async fn run_migrations(
    pool: &MySqlPool,
    migrations: &[(&str, &str)],
) -> Result<MigrationReport, MigrateError> {
    ensure_schema_migration_table(pool).await?;

    let applied_versions = list_applied_versions(pool).await?;
    let mut report = MigrationReport {
        applied: Vec::new(),
        baselined: Vec::new(),
        already_applied: Vec::new(),
    };

    if applied_versions.is_empty() && schema_already_current(pool).await? {
        for (version, _) in migrations {
            record_migration(pool, version).await?;
            report.baselined.push((*version).to_string());
        }
        return Ok(report);
    }

    for (version, sql) in migrations {
        if applied_versions.iter().any(|applied| applied == version) {
            report.already_applied.push((*version).to_string());
            continue;
        }

        execute_sql_script(pool, sql).await.map_err(|error| {
            MigrateError::InvalidMigration(format!("migration {version} failed: {error}"))
        })?;
        record_migration(pool, version).await?;
        report.applied.push((*version).to_string());
    }

    Ok(report)
}

pub async fn migration_status(
    pool: &MySqlPool,
    migrations: &[(&str, &str)],
) -> Result<Vec<(String, bool)>, MigrateError> {
    ensure_schema_migration_table(pool).await?;
    let applied_versions = list_applied_versions(pool).await?;
    Ok(migrations
        .iter()
        .map(|(version, _)| {
            (
                (*version).to_string(),
                applied_versions.iter().any(|applied| applied == version),
            )
        })
        .collect())
}

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

pub(crate) fn split_sql_statements(sql: &str) -> Vec<String> {
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

async fn ensure_schema_migration_table(pool: &MySqlPool) -> Result<(), MigrateError> {
    if table_exists(pool, "SchemaMigration").await? {
        return Ok(());
    }

    sqlx::query(
        "CREATE TABLE SchemaMigration (
            version VARCHAR(64) PRIMARY KEY,
            appliedAt TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await
    .map_err(|error| {
        MigrateError::InvalidMigration(format!(
            "SchemaMigration table is missing and CREATE was denied ({error}). \
             Reload createDatabase.sql or grant CREATE/ALTER to the app user, \
             then re-run migrate."
        ))
    })?;
    Ok(())
}

async fn list_applied_versions(pool: &MySqlPool) -> Result<Vec<String>, MigrateError> {
    let rows = sqlx::query("SELECT version FROM SchemaMigration ORDER BY version")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("version"))
        .collect())
}

async fn record_migration(pool: &MySqlPool, version: &str) -> Result<(), MigrateError> {
    sqlx::query("INSERT INTO SchemaMigration (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
}

async fn execute_sql_script(pool: &MySqlPool, sql: &str) -> Result<(), MigrateError> {
    for statement in split_sql_statements(sql) {
        sqlx::query(&statement).execute(pool).await?;
    }
    Ok(())
}

async fn schema_already_current(pool: &MySqlPool) -> Result<bool, MigrateError> {
    let user_table_exists = table_exists(pool, "User").await?;
    if !user_table_exists {
        return Ok(false);
    }

    let has_scan_speed = column_exists(pool, "Robot", "scanSpeed").await?;
    let has_scan_time = column_exists(pool, "Robot", "scanTime").await?;
    let has_session_version = column_exists(pool, "User", "sessionVersion").await?;
    Ok(!has_scan_speed && has_scan_time && has_session_version)
}

async fn table_exists(pool: &MySqlPool, table_name: &str) -> Result<bool, MigrateError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = DATABASE() AND table_name = ?",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

async fn column_exists(
    pool: &MySqlPool,
    table_name: &str,
    column_name: &str,
) -> Result<bool, MigrateError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_schema = DATABASE()
           AND table_name = ?
           AND column_name = ?",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::{EMBEDDED_MIGRATIONS, load_migrations_from_dir, split_sql_statements};

    #[test]
    fn embedded_migrations_match_filesystem() {
        let dir = super::default_migrations_dir();
        let from_disk = load_migrations_from_dir(&dir).expect("load migrations dir");
        assert_eq!(from_disk.len(), EMBEDDED_MIGRATIONS.len());
        for ((disk_version, disk_sql), (embedded_version, embedded_sql)) in
            from_disk.iter().zip(EMBEDDED_MIGRATIONS.iter())
        {
            assert_eq!(disk_version, embedded_version);
            assert_eq!(
                split_sql_statements(disk_sql),
                split_sql_statements(embedded_sql)
            );
        }
    }

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
}
