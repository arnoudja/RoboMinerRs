use sqlx::MySqlPool;

use crate::users::{touch_user_last_login_time, user_exists};
use crate::{
    CreateProgramSourceRequest, CreatedProgramSource, DbOutcome, ProgramSourceWriteRejection,
    ProgramSourceWriteRequest, db_ok, db_reject,
};

pub async fn create_program_source(
    pool: &MySqlPool,
    request: CreateProgramSourceRequest,
) -> Result<DbOutcome<CreatedProgramSource, ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if let Some(rejection) =
        validate_program_source_write(&request.source_name, &request.source_code)
    {
        transaction.rollback().await?;
        return db_reject(rejection);
    }

    if !user_exists(&mut transaction, request.user_id).await? {
        transaction.rollback().await?;
        return db_reject(ProgramSourceWriteRejection::UnknownUser);
    }

    let result = sqlx::query!(
        "INSERT INTO ProgramSource \
         (userId, sourceName, sourceCode, verified, compiledSize, errorDescription) \
         VALUES (?, ?, ?, false, -1, '')",
        request.user_id,
        request.source_name,
        request.source_code,
    )
    .execute(&mut *transaction)
    .await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;

    db_ok(CreatedProgramSource {
        program_source_id: result.last_insert_id() as i64,
    })
}

pub async fn delete_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM ProgramSource WHERE id = ?", program_source_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_program_source(
    pool: &MySqlPool,
    request: ProgramSourceWriteRequest,
) -> Result<DbOutcome<(), ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if let Some(rejection) =
        validate_program_source_write(&request.source_name, &request.source_code)
    {
        transaction.rollback().await?;
        return db_reject(rejection);
    }

    if !program_source_belongs_to_user(&mut transaction, request.program_source_id, request.user_id)
        .await?
    {
        transaction.rollback().await?;
        return db_reject(ProgramSourceWriteRejection::UnknownProgramSource);
    }

    sqlx::query!(
        "UPDATE ProgramSource \
         SET sourceName = ?, sourceCode = ?, verified = false \
         WHERE id = ? AND userId = ?",
        request.source_name,
        request.source_code,
        request.program_source_id,
        request.user_id,
    )
    .execute(&mut *transaction)
    .await?;

    touch_user_last_login_time(&mut transaction, request.user_id).await?;

    transaction.commit().await?;
    db_ok(())
}

pub async fn delete_program_source_for_user(
    pool: &MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<DbOutcome<(), ProgramSourceWriteRejection>, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    if !program_source_belongs_to_user(&mut transaction, program_source_id, user_id).await? {
        transaction.rollback().await?;
        return db_reject(ProgramSourceWriteRejection::UnknownProgramSource);
    }

    if program_source_robot_count(&mut transaction, program_source_id).await? > 0 {
        transaction.rollback().await?;
        return db_reject(ProgramSourceWriteRejection::SourceInUse);
    }

    sqlx::query!(
        "DELETE FROM ProgramSource WHERE id = ? AND userId = ?",
        program_source_id,
        user_id
    )
    .execute(&mut *transaction)
    .await?;

    touch_user_last_login_time(&mut transaction, user_id).await?;

    transaction.commit().await?;
    db_ok(())
}

pub async fn set_valid_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
    compiled_size: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE ProgramSource \
         SET errorDescription = '', verified = true, compiledSize = ? \
         WHERE id = ?",
        compiled_size,
        program_source_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_invalid_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
    error_description: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE ProgramSource \
         SET errorDescription = ?, verified = false, compiledSize = -1 \
         WHERE id = ?",
        error_description,
        program_source_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn program_source_belongs_to_user(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    program_source_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM ProgramSource WHERE id = ? AND userId = ? FOR UPDATE")
            .bind(program_source_id)
            .bind(user_id)
            .fetch_optional(&mut **transaction)
            .await?;

    Ok(exists.is_some())
}

pub(crate) async fn program_source_robot_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    program_source_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM Robot WHERE programSourceId = ?")
        .bind(program_source_id)
        .fetch_one(&mut **transaction)
        .await
}

/// Maximum accepted robot program source length (UTF-8 bytes).
pub const MAX_PROGRAM_SOURCE_CODE_BYTES: usize = 16_384;

pub(crate) fn validate_program_source_write(
    source_name: &str,
    source_code: &str,
) -> Option<ProgramSourceWriteRejection> {
    if source_name.is_empty() {
        return Some(ProgramSourceWriteRejection::EmptySourceName);
    }
    if source_code.is_empty() {
        return Some(ProgramSourceWriteRejection::EmptySourceCode);
    }
    if source_code.len() > MAX_PROGRAM_SOURCE_CODE_BYTES {
        return Some(ProgramSourceWriteRejection::SourceCodeTooLong);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{MAX_PROGRAM_SOURCE_CODE_BYTES, validate_program_source_write};
    use crate::ProgramSourceWriteRejection;

    #[test]
    fn validate_program_source_write_requires_name_and_code() {
        assert_eq!(
            validate_program_source_write("", "mine();"),
            Some(ProgramSourceWriteRejection::EmptySourceName)
        );
        assert_eq!(
            validate_program_source_write("main", ""),
            Some(ProgramSourceWriteRejection::EmptySourceCode)
        );
        assert_eq!(validate_program_source_write("main", "mine();"), None);
    }

    #[test]
    fn validate_program_source_write_rejects_oversized_source() {
        let oversized = "a".repeat(MAX_PROGRAM_SOURCE_CODE_BYTES + 1);
        assert_eq!(
            validate_program_source_write("main", &oversized),
            Some(ProgramSourceWriteRejection::SourceCodeTooLong)
        );
        let at_limit = "a".repeat(MAX_PROGRAM_SOURCE_CODE_BYTES);
        assert_eq!(validate_program_source_write("main", &at_limit), None);
    }
}
