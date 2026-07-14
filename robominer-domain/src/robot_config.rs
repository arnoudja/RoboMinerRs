use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_robot_config_program_sources(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::ProgramSourceRecord>, DomainError> {
    robominer_db::list_program_sources_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_program_source_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::ProgramSourceStateRecord>, DomainError> {
    robominer_db::list_program_source_states_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_robot_config_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::RobotConfigStateRecord>, DomainError> {
    robominer_db::list_robot_config_states(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_robot_config_part_asset_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::RobotConfigPartAssetStateRecord>, DomainError> {
    robominer_db::list_robot_config_part_asset_states(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn update_robot_config(
    pool: &MySqlPool,
    request: robominer_db::UpdateRobotConfigRequest,
) -> Result<
    Result<robominer_db::UpdatedRobotConfig, robominer_db::UpdateRobotConfigRejection>,
    DomainError,
> {
    robominer_db::update_robot_config(pool, request)
        .await
        .map_err(DomainError::Database)
}

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

pub async fn apply_program_source_to_linked_robots(
    pool: &MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<robominer_db::AppliedProgramSource, DomainError> {
    robominer_db::apply_verified_program_source_to_idle_robots(pool, user_id, program_source_id)
        .await
        .map_err(DomainError::Database)
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

pub async fn delete_program_source(
    pool: &MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<Result<(), robominer_db::ProgramSourceWriteRejection>, DomainError> {
    robominer_db::delete_program_source_for_user(pool, user_id, program_source_id)
        .await
        .map_err(DomainError::Database)
}
