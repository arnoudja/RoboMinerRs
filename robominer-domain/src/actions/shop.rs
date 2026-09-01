use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuyRobotPartOutcome {
    Success(robominer_db::RobotPartTransaction),
    Rejected(robominer_db::RobotPartTransactionRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SellRobotPartOutcome {
    Success(robominer_db::RobotPartTransaction),
    Rejected(robominer_db::RobotPartTransactionRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SellAllUnassignedRobotPartsOutcome {
    Success(robominer_db::SellAllUnassignedRobotPartsResult),
    Rejected(robominer_db::RobotPartTransactionRejection),
}

pub async fn buy_robot_part(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<BuyRobotPartOutcome, DomainError> {
    match robominer_db::buy_robot_part(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(BuyRobotPartOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(BuyRobotPartOutcome::Rejected(rejection))
        }
    }
}

pub async fn sell_robot_part(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<SellRobotPartOutcome, DomainError> {
    match robominer_db::sell_robot_part(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(SellRobotPartOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(SellRobotPartOutcome::Rejected(rejection))
        }
    }
}

pub async fn sell_all_unassigned_robot_parts(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<SellAllUnassignedRobotPartsOutcome, DomainError> {
    match robominer_db::sell_all_unassigned_robot_parts(pool, user_id).await? {
        robominer_db::DbOutcome::Success(value) => {
            Ok(SellAllUnassignedRobotPartsOutcome::Success(value))
        }
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(SellAllUnassignedRobotPartsOutcome::Rejected(rejection))
        }
    }
}
