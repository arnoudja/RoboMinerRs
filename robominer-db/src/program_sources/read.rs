use sqlx::MySqlPool;

use crate::mappers::{ProgramSourceRow, program_source_rows, program_source_state_record};
use crate::{ProgramSourceRecord, ProgramSourceVerification};

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
    .map(program_source_rows)
}

pub async fn list_program_source_states_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<crate::ProgramSourceStateRecord>, sqlx::Error> {
    let rows = sqlx::query(
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
    .await?;

    rows.into_iter().map(program_source_state_record).collect()
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
