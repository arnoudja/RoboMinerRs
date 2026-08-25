//! View-model mapping for the mining queue page (records → display items).

use std::collections::HashMap;

use super::MiningQueueDisplayItem;

pub(super) async fn load_mining_queue_display_items(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueueDisplayItem>, crate::page_context::PageLoadError> {
    let items = robominer_db::list_mining_queue_page_items(pool, user_id).await?;
    let states = robominer_db::list_mining_queue_states_for_user(pool, user_id).await?;
    let state_map: HashMap<i64, robominer_db::MiningQueueStateRecord> = states
        .into_iter()
        .map(|state| (state.mining_queue_id, state))
        .collect();

    Ok(items
        .into_iter()
        .map(|item| {
            let state = state_map.get(&item.mining_queue_id);
            MiningQueueDisplayItem {
                mining_queue_id: item.mining_queue_id,
                robot_id: item.robot_id,
                mining_area_id: item.mining_area_id,
                area_name: item.area_name,
                rally_result_id: item.rally_result_id,
                status: state
                    .map(|state| state.status)
                    .unwrap_or(robominer_db::MiningQueueStatus::Queued),
                time_left_seconds: state
                    .map(|state| state.time_left_seconds)
                    .unwrap_or_default(),
            }
        })
        .collect())
}
