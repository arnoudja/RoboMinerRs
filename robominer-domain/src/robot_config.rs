use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn create_program_source(
    pool: &MySqlPool,
    request: robominer_db::CreateProgramSourceRequest,
) -> Result<
    Result<robominer_db::CreatedProgramSource, robominer_db::ProgramSourceWriteRejection>,
    DomainError,
> {
    let source_code = request.source_code.clone();
    let result = robominer_db::create_program_source(pool, request)
        .await
        .map_err(DomainError::Database)?;
    if let Ok(created) = &result {
        verify_and_mark_program_source(pool, created.program_source_id, &source_code).await?;
    }
    Ok(result)
}

pub async fn update_program_source(
    pool: &MySqlPool,
    request: robominer_db::ProgramSourceWriteRequest,
) -> Result<Result<(), robominer_db::ProgramSourceWriteRejection>, DomainError> {
    let program_source_id = request.program_source_id;
    let source_code = request.source_code.clone();
    let result = robominer_db::update_program_source(pool, request)
        .await
        .map_err(DomainError::Database)?;
    if result.is_ok() {
        verify_and_mark_program_source(pool, program_source_id, &source_code).await?;
    }
    Ok(result)
}

async fn verify_and_mark_program_source(
    pool: &MySqlPool,
    program_source_id: i64,
    source_code: &str,
) -> Result<(), DomainError> {
    let verification = robominer_program::verify_source(source_code);
    if verification.verified {
        robominer_db::set_valid_program_source(pool, program_source_id, verification.compiled_size)
            .await
            .map_err(DomainError::Database)
    } else {
        robominer_db::set_invalid_program_source(
            pool,
            program_source_id,
            &verification.error_description,
        )
        .await
        .map_err(DomainError::Database)
    }
}
