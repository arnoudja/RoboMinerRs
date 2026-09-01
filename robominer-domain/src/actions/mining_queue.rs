use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueMiningOutcome {
    Success(robominer_db::EnqueuedMining),
    Rejected(robominer_db::EnqueueMiningRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelMiningQueueOutcome {
    Success(robominer_db::CanceledMiningQueue),
    Rejected(robominer_db::CancelMiningQueueRejection),
}

pub async fn enqueue_mining(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::EnqueueMiningRequest,
) -> Result<EnqueueMiningOutcome, DomainError> {
    match robominer_db::enqueue_mining(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(EnqueueMiningOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(EnqueueMiningOutcome::Rejected(rejection))
        }
    }
}

pub async fn cancel_mining_queue(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::CancelMiningQueueRequest,
) -> Result<CancelMiningQueueOutcome, DomainError> {
    match robominer_db::cancel_mining_queue(pool, request).await? {
        robominer_db::DbOutcome::Success(value) => Ok(CancelMiningQueueOutcome::Success(value)),
        robominer_db::DbOutcome::Rejected(rejection) => {
            Ok(CancelMiningQueueOutcome::Rejected(rejection))
        }
    }
}
