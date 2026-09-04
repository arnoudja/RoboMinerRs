use sqlx::MySqlPool;
use sqlx::Row;

use super::errors::MigrateError;

pub(super) async fn schema_already_current(pool: &MySqlPool) -> Result<bool, MigrateError> {
    // Checklist must cover every migration represented by createDatabase.sql /
    // resources/database/migrations/*.sql. When adding a migration that also
    // lands in createDatabase.sql, add a matching probe here so empty
    // SchemaMigration tables are not silently baselined without that change.
    let user_table_exists = table_exists(pool, "User").await?;
    if !user_table_exists {
        return Ok(false);
    }

    let has_scan_speed = column_exists(pool, "Robot", "scanSpeed").await?;
    let has_scan_time = column_exists(pool, "Robot", "scanTime").await?;
    let has_session_version = column_exists(pool, "User", "sessionVersion").await?;
    let has_score_ore_target = column_exists(pool, "MiningArea", "scoreOreTarget").await?;
    let has_ai_robot = table_exists(pool, "AIRobot").await?;
    let has_depot_tax_rate = column_exists(pool, "MiningArea", "depotTaxRate").await?;
    let has_depot_amount = column_exists(pool, "MiningOreResult", "depotAmount").await?;
    let has_lifetime_total_runs =
        column_exists(pool, "MiningAreaLifetimeResult", "totalRuns").await?;
    let has_depot_total_requirement =
        table_exists(pool, "AchievementStepDepotTotalRequirement").await?;
    let has_processing_lease = column_exists(pool, "MiningQueue", "processingLeaseUntil").await?;
    let has_lifetime_depot_amount =
        column_exists(pool, "RobotLifetimeResult", "depotAmount").await?;
    // Migration 012: claimable wallet-index on MiningQueue.
    let has_claimable_index =
        index_exists(pool, "MiningQueue", "idx_mining_queue_claimable").await?;
    Ok(!has_scan_speed
        && has_scan_time
        && has_session_version
        && has_score_ore_target
        && has_ai_robot
        && has_depot_tax_rate
        && has_depot_amount
        && has_lifetime_total_runs
        && has_depot_total_requirement
        && has_processing_lease
        && has_lifetime_depot_amount
        && has_claimable_index)
}

pub(super) async fn ensure_schema_migration_table(pool: &MySqlPool) -> Result<(), MigrateError> {
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

pub(super) async fn list_applied_versions(pool: &MySqlPool) -> Result<Vec<String>, MigrateError> {
    let rows = sqlx::query("SELECT version FROM SchemaMigration ORDER BY version")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("version"))
        .collect())
}

pub(super) async fn record_migration(pool: &MySqlPool, version: &str) -> Result<(), MigrateError> {
    sqlx::query("INSERT INTO SchemaMigration (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
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

async fn index_exists(
    pool: &MySqlPool,
    table_name: &str,
    index_name: &str,
) -> Result<bool, MigrateError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.statistics
         WHERE table_schema = DATABASE()
           AND table_name = ?
           AND index_name = ?",
    )
    .bind(table_name)
    .bind(index_name)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}
