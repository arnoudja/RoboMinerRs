//! View-model mapping for the mining queue page (records → display items).

use std::collections::HashMap;

use super::{
    MiningQueueAreaCostView, MiningQueueAreaSupplyView, MiningQueueAreaView,
    MiningQueueAssetSummaryView, MiningQueueDisplayItem, MiningQueueOreAssetView,
    MiningQueueRobotView, MiningQueueScoreView,
};

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

pub(super) fn asset_summary_view(
    record: robominer_db::UserAssetSummaryRecord,
) -> MiningQueueAssetSummaryView {
    MiningQueueAssetSummaryView {
        mining_queue_size: record.mining_queue_size,
    }
}

pub(super) fn ore_asset_view(
    record: robominer_db::UserOreAssetStateRecord,
) -> MiningQueueOreAssetView {
    MiningQueueOreAssetView {
        ore_id: record.ore_id,
        ore_name: record.ore_name,
        amount: record.amount,
        max_allowed: record.max_allowed,
        depot_max_allowed: record.depot_max_allowed,
    }
}

pub(super) fn robot_view(record: robominer_db::MiningQueuePageRobotRecord) -> MiningQueueRobotView {
    MiningQueueRobotView {
        robot_id: record.robot_id,
        robot_name: record.robot_name,
        recharge_time: record.recharge_time,
    }
}

pub(super) fn area_view(record: robominer_db::MiningQueuePageAreaRecord) -> MiningQueueAreaView {
    MiningQueueAreaView {
        mining_area_id: record.mining_area_id,
        area_name: record.area_name,
        tax_rate: record.tax_rate,
        depot_tax_rate: record.depot_tax_rate,
        mining_time: record.mining_time,
        max_moves: record.max_moves,
        size_x: record.size_x,
        size_y: record.size_y,
        score_ore_target: record.score_ore_target,
    }
}

pub(super) fn area_cost_view(
    record: robominer_db::MiningQueuePageAreaCostRecord,
) -> MiningQueueAreaCostView {
    MiningQueueAreaCostView {
        mining_area_id: record.mining_area_id,
        ore_id: record.ore_id,
        ore_name: record.ore_name,
        amount: record.amount,
    }
}

pub(super) fn area_supply_view(
    record: robominer_db::MiningQueuePageAreaSupplyRecord,
) -> MiningQueueAreaSupplyView {
    MiningQueueAreaSupplyView {
        mining_area_id: record.mining_area_id,
        ore_id: record.ore_id,
        ore_name: record.ore_name,
        supply: record.supply,
        radius: record.radius,
    }
}

pub(super) fn score_view(record: robominer_db::RobotMiningAreaScoreRecord) -> MiningQueueScoreView {
    MiningQueueScoreView {
        robot_id: record.robot_id,
        mining_area_id: record.mining_area_id,
        score: record.score,
    }
}
