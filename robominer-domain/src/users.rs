use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn create_user(
    pool: &MySqlPool,
    request: robominer_db::CreateUserRequest,
) -> Result<Result<robominer_db::CreatedUser, robominer_db::CreateUserRejection>, DomainError> {
    robominer_db::create_user(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn update_user_account(
    pool: &MySqlPool,
    request: robominer_db::UpdateUserAccountRequest,
) -> Result<
    Result<robominer_db::UpdatedUserAccount, robominer_db::UpdateUserAccountRejection>,
    DomainError,
> {
    robominer_db::update_user_account(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn verify_login(
    pool: &MySqlPool,
    request: robominer_db::VerifyLoginRequest,
) -> Result<Result<robominer_db::VerifiedLogin, robominer_db::VerifyLoginRejection>, DomainError> {
    robominer_db::verify_login(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn verify_user_password(
    pool: &MySqlPool,
    request: robominer_db::VerifyUserPasswordRequest,
) -> Result<Result<robominer_db::VerifiedLogin, robominer_db::VerifyLoginRejection>, DomainError> {
    robominer_db::verify_user_password(pool, request)
        .await
        .map_err(DomainError::Database)
}
