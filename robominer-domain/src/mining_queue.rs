use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn claim_user_results(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<robominer_db::ClaimedUserResults, DomainError> {
    robominer_db::claim_user_results(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn enqueue_mining(
    pool: &MySqlPool,
    request: robominer_db::EnqueueMiningRequest,
) -> Result<Result<robominer_db::EnqueuedMining, robominer_db::EnqueueMiningRejection>, DomainError>
{
    robominer_db::enqueue_mining(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn cancel_mining_queue(
    pool: &MySqlPool,
    request: robominer_db::CancelMiningQueueRequest,
) -> Result<
    Result<robominer_db::CanceledMiningQueue, robominer_db::CancelMiningQueueRejection>,
    DomainError,
> {
    robominer_db::cancel_mining_queue(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueueStateRecord>, DomainError> {
    robominer_db::list_mining_queue_states_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_robots(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageRobotRecord>, DomainError> {
    robominer_db::list_mining_queue_page_robots(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_areas(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageAreaRecord>, DomainError> {
    robominer_db::list_mining_queue_page_areas(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_area_costs(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageAreaCostRecord>, DomainError> {
    robominer_db::list_mining_queue_page_area_costs(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_area_supplies(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageAreaSupplyRecord>, DomainError> {
    robominer_db::list_mining_queue_page_area_supplies(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_area_yields(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageAreaYieldRecord>, DomainError> {
    robominer_db::list_mining_queue_page_area_yields(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_queue_page_items(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::MiningQueuePageItemRecord>, DomainError> {
    robominer_db::list_mining_queue_page_items(pool, user_id)
        .await
        .map_err(DomainError::Database)
}
