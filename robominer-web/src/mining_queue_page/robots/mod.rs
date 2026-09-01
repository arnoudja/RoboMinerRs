use std::collections::HashMap;

use crate::html::format_ore_shortfall;
use crate::mining_queue_page::{MiningQueueAreaCostView, MiningQueuePageState};

mod card;
mod queue_row;
mod wallet;

pub(in crate::mining_queue_page) use card::render_robot_card;
pub(in crate::mining_queue_page) use queue_row::format_queue_time_left;
pub(in crate::mining_queue_page) use wallet::render_wallet_strip;

pub(super) fn enqueue_block_reason(
    state: &MiningQueuePageState,
    queue_len: usize,
    selected_area_id: i64,
    cost_map: &HashMap<i64, Vec<&MiningQueueAreaCostView>>,
    ore_amount_map: &HashMap<i64, i32>,
) -> Option<String> {
    if queue_len as i64 >= i64::from(state.asset_summary.mining_queue_size) {
        return Some("Queue full for this robot.".to_string());
    }
    if !state
        .areas
        .iter()
        .any(|area| area.mining_area_id == selected_area_id)
    {
        return Some("Mining area not available.".to_string());
    }
    for cost in cost_map.get(&selected_area_id).into_iter().flatten() {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return Some(format_ore_shortfall(need, &cost.ore_name));
        }
    }
    None
}
