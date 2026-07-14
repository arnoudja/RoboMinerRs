use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn list_mining_result_states(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<robominer_db::MiningResultStateRecord>, DomainError> {
    robominer_db::list_mining_result_states_for_user(pool, user_id, maximum_results)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_result_ore_states(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<robominer_db::MiningResultOreStateRecord>, DomainError> {
    robominer_db::list_mining_result_ore_states_for_user(pool, user_id, maximum_results)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_mining_result_action_states(
    pool: &MySqlPool,
    user_id: i64,
    maximum_results: i64,
) -> Result<Vec<robominer_db::MiningResultActionStateRecord>, DomainError> {
    robominer_db::list_mining_result_action_states_for_user(pool, user_id, maximum_results)
        .await
        .map_err(DomainError::Database)
}
