use crate::assert_sql_safe;
use sqlx::MySqlPool;

use super::errors::MigrateError;
use super::loader::split_sql_statements;

/// sqlx cannot run PREPARE/EXECUTE over the MySQL binary protocol (error 1295),
/// so dynamic DDL for migration 006 lives here instead of in the SQL file.
pub(super) async fn prepare_ai_robot_table_migration(pool: &MySqlPool) -> Result<(), MigrateError> {
    let mut conn = pool.acquire().await?;
    drop_column_foreign_key(&mut conn, "MiningArea", "aiRobotId").await?;
    ensure_nullable_int_column(&mut conn, "MiningArea", "aiRobotIdNew").await?;
    Ok(())
}

async fn drop_column_foreign_key(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
    table_name: &str,
    column_name: &str,
) -> Result<(), MigrateError> {
    let constraint_name: Option<String> = sqlx::query_scalar(
        "SELECT CONSTRAINT_NAME \
         FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = DATABASE() \
           AND TABLE_NAME = ? \
           AND COLUMN_NAME = ? \
           AND REFERENCED_TABLE_NAME IS NOT NULL \
         LIMIT 1",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_optional(&mut **conn)
    .await?;

    let Some(constraint_name) = constraint_name else {
        return Ok(());
    };
    if !is_safe_sql_identifier(&constraint_name) || !is_safe_sql_identifier(table_name) {
        return Err(MigrateError::InvalidMigration(format!(
            "refusing to drop foreign key with unsafe identifier ({table_name}.{constraint_name})"
        )));
    }

    let sql = format!("ALTER TABLE `{table_name}` DROP FOREIGN KEY `{constraint_name}`");
    sqlx::query(assert_sql_safe(sql))
        .execute(&mut **conn)
        .await?;
    Ok(())
}

async fn ensure_nullable_int_column(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
    table_name: &str,
    column_name: &str,
) -> Result<(), MigrateError> {
    if column_exists_on_conn(conn, table_name, column_name).await? {
        return Ok(());
    }
    if !is_safe_sql_identifier(table_name) || !is_safe_sql_identifier(column_name) {
        return Err(MigrateError::InvalidMigration(format!(
            "refusing to add column with unsafe identifier ({table_name}.{column_name})"
        )));
    }

    let sql = format!("ALTER TABLE `{table_name}` ADD COLUMN `{column_name}` INT NULL");
    sqlx::query(assert_sql_safe(sql))
        .execute(&mut **conn)
        .await?;
    Ok(())
}

async fn column_exists_on_conn(
    conn: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
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
    .fetch_one(&mut **conn)
    .await?;
    Ok(count > 0)
}

fn is_safe_sql_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(super) async fn execute_sql_script(pool: &MySqlPool, sql: &str) -> Result<(), MigrateError> {
    // Keep one connection for the whole script so statement order is preserved
    // on a single session.
    let mut conn = pool.acquire().await?;
    for statement in split_sql_statements(sql) {
        sqlx::query(assert_sql_safe(statement))
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}
