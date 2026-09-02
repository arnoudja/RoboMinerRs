use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateUserAccountOutcome {
    Success(robominer_db::UpdatedUserAccount),
    Rejected(robominer_db::UpdateUserAccountRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoutAllDevicesOutcome {
    Success { session_version: i32 },
    UnknownUser,
}

pub async fn update_user_account(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::UpdateUserAccountRequest,
) -> Result<UpdateUserAccountOutcome, DomainError> {
    match robominer_db::update_user_account(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(UpdateUserAccountOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(UpdateUserAccountOutcome::Rejected(rejection))
        }
    }
}

pub async fn logout_all_devices(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<LogoutAllDevicesOutcome, DomainError> {
    match robominer_db::bump_user_session_version(pool, user_id).await? {
        Some(session_version) => Ok(LogoutAllDevicesOutcome::Success { session_version }),
        None => Ok(LogoutAllDevicesOutcome::UnknownUser),
    }
}
