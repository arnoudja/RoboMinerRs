use sqlx::MySqlPool;

use crate::{ProgramSourceRecord, ProgramSourceStateRecord, ProgramSourceVerification};

#[derive(sqlx::FromRow)]
pub(crate) struct ProgramSourceRow {
    id: i64,
    #[sqlx(rename = "userId")]
    user_id: i64,
    #[sqlx(rename = "sourceName")]
    source_name: String,
    #[sqlx(rename = "sourceCode")]
    source_code: Option<String>,
    verified: bool,
    #[sqlx(rename = "compiledSize")]
    compiled_size: i32,
    #[sqlx(rename = "errorDescription")]
    error_description: Option<String>,
}

impl From<ProgramSourceRow> for ProgramSourceRecord {
    fn from(row: ProgramSourceRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            source_name: row.source_name,
            source_code: row.source_code,
            verified: row.verified,
            compiled_size: row.compiled_size,
            error_description: row.error_description.unwrap_or_default(),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProgramSourceStateRow {
    id: i64,
    #[sqlx(rename = "userId")]
    user_id: i64,
    #[sqlx(rename = "sourceName")]
    source_name: String,
    #[sqlx(rename = "sourceCode")]
    source_code: Option<String>,
    verified: bool,
    #[sqlx(rename = "compiledSize")]
    compiled_size: i32,
    #[sqlx(rename = "errorDescription")]
    error_description: Option<String>,
    #[sqlx(rename = "linkedRobotCount")]
    linked_robot_count: i64,
}

impl From<ProgramSourceStateRow> for ProgramSourceStateRecord {
    fn from(row: ProgramSourceStateRow) -> Self {
        Self {
            source: ProgramSourceRecord {
                id: row.id,
                user_id: row.user_id,
                source_name: row.source_name,
                source_code: row.source_code,
                verified: row.verified,
                compiled_size: row.compiled_size,
                error_description: row.error_description.unwrap_or_default(),
            },
            linked_robot_count: row.linked_robot_count,
        }
    }
}

pub async fn get_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT sourceCode FROM ProgramSource WHERE id = ?")
        .bind(program_source_id)
        .fetch_optional(pool)
        .await
}

pub async fn list_program_sources_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<ProgramSourceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ProgramSourceRow>(
        "SELECT id, userId, sourceName, sourceCode, verified, compiledSize, errorDescription \
         FROM ProgramSource \
         WHERE userId = ? \
         ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(ProgramSourceRecord::from).collect())
}

pub async fn list_program_source_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<ProgramSourceStateRecord>, sqlx::Error> {
    sqlx::query_as::<_, ProgramSourceStateRow>(
        "SELECT ProgramSource.id, ProgramSource.userId, ProgramSource.sourceName, \
                ProgramSource.sourceCode, ProgramSource.verified, ProgramSource.compiledSize, \
                ProgramSource.errorDescription, \
                (SELECT COUNT(*) FROM Robot WHERE Robot.programSourceId = ProgramSource.id) \
                  AS linkedRobotCount \
         FROM ProgramSource \
         WHERE ProgramSource.userId = ? \
         ORDER BY ProgramSource.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(ProgramSourceStateRecord::from)
            .collect()
    })
}

pub async fn get_program_source_verification(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<Option<ProgramSourceVerification>, sqlx::Error> {
    sqlx::query_as::<_, (bool, i32, Option<String>)>(
        "SELECT verified, compiledSize, errorDescription \
         FROM ProgramSource \
         WHERE id = ?",
    )
    .bind(program_source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(verified, compiled_size, error_description)| ProgramSourceVerification {
                verified,
                compiled_size,
                error_description: error_description.unwrap_or_default(),
            },
        )
    })
}
