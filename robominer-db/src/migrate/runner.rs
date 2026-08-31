use sqlx::MySqlPool;

use super::errors::MigrateError;
use super::schema::{
    ensure_schema_migration_table, list_applied_versions, record_migration, schema_already_current,
};
use super::special::{execute_sql_script, prepare_ai_robot_table_migration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub applied: Vec<String>,
    pub baselined: Vec<String>,
    pub already_applied: Vec<String>,
}

const MIGRATION_LOCK_NAME: &str = "robominer_schema_migrate";
const MIGRATION_LOCK_TIMEOUT_SECONDS: i32 = 60;

pub async fn run_embedded_migrations(pool: &MySqlPool) -> Result<MigrationReport, MigrateError> {
    run_migrations(pool, super::EMBEDDED_MIGRATIONS).await
}

pub async fn run_migrations_from_dir(
    pool: &MySqlPool,
    migrations_dir: &std::path::Path,
) -> Result<MigrationReport, MigrateError> {
    let migrations = super::loader::load_migrations_from_dir(migrations_dir)?;
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
    // Serialize migrators across web/engine startups so two processes cannot
    // interleave DDL on the same schema. GET_LOCK is connection-scoped, so the
    // lock connection is held for the whole migrate.
    let mut lock_conn = pool.acquire().await?;
    acquire_migration_lock(&mut lock_conn).await?;
    let result = run_migrations_locked(pool, migrations).await;
    release_migration_lock(&mut lock_conn).await?;
    result
}

async fn run_migrations_locked(
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

        if *version == "006_ai_robot_table" {
            prepare_ai_robot_table_migration(pool)
                .await
                .map_err(|error| {
                    MigrateError::InvalidMigration(format!("migration {version} failed: {error}"))
                })?;
        }

        execute_sql_script(pool, sql).await.map_err(|error| {
            MigrateError::InvalidMigration(format!("migration {version} failed: {error}"))
        })?;
        record_migration(pool, version).await?;
        report.applied.push((*version).to_string());
    }

    Ok(report)
}

async fn acquire_migration_lock(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
) -> Result<(), MigrateError> {
    let acquired: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(MIGRATION_LOCK_NAME)
        .bind(MIGRATION_LOCK_TIMEOUT_SECONDS)
        .fetch_one(&mut **conn)
        .await?;
    if acquired != Some(1) {
        return Err(MigrateError::InvalidMigration(format!(
            "could not acquire migration lock {MIGRATION_LOCK_NAME} within {MIGRATION_LOCK_TIMEOUT_SECONDS}s"
        )));
    }
    Ok(())
}

async fn release_migration_lock(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
) -> Result<(), MigrateError> {
    let _: Option<i64> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(MIGRATION_LOCK_NAME)
        .fetch_one(&mut **conn)
        .await?;
    Ok(())
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
