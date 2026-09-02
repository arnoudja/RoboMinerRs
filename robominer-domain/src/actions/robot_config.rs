use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateRobotConfigOutcome {
    Success(robominer_db::UpdatedRobotConfig),
    Rejected(robominer_db::UpdateRobotConfigRejection),
}

pub async fn update_robot_config(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::UpdateRobotConfigRequest,
) -> Result<UpdateRobotConfigOutcome, DomainError> {
    match robominer_db::update_robot_config(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(UpdateRobotConfigOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(UpdateRobotConfigOutcome::Rejected(rejection))
        }
    }
}
