//! View-model mapping for the shop page (records → page-local views).

use super::{CatalogPartView, OreAssetView, OreView, PartCostView, PartStateView, PartTypeView};

pub(super) fn ore_view(id: i64, ore_name: String) -> OreView {
    OreView { id, ore_name }
}

pub(super) fn part_type_view(record: robominer_db::RobotPartTypeRecord) -> PartTypeView {
    PartTypeView {
        id: record.id,
        type_name: record.type_name,
    }
}

pub(super) fn catalog_part_view(
    record: robominer_db::ShopRobotPartCatalogRecord,
) -> CatalogPartView {
    CatalogPartView {
        robot_part_id: record.robot_part_id,
        type_id: record.type_id,
        tier_id: record.tier_id,
        tier_name: record.tier_name,
        part_name: record.part_name,
        ore_capacity: record.ore_capacity,
        mining_capacity: record.mining_capacity,
        battery_capacity: record.battery_capacity,
        memory_capacity: record.memory_capacity,
        cpu_capacity: record.cpu_capacity,
        forward_capacity: record.forward_capacity,
        backward_capacity: record.backward_capacity,
        rotate_capacity: record.rotate_capacity,
        recharge_time: record.recharge_time,
        scan_time: record.scan_time,
        scan_distance: record.scan_distance,
        weight: record.weight,
        volume: record.volume,
        power_usage: record.power_usage,
    }
}

pub(super) fn part_cost_view(record: robominer_db::ShopRobotPartCostRecord) -> PartCostView {
    PartCostView {
        robot_part_id: record.robot_part_id,
        ore_id: record.ore_id,
        ore_name: record.ore_name,
        amount: record.amount,
    }
}

pub(super) fn part_state_view(record: robominer_db::ShopRobotPartStateRecord) -> PartStateView {
    PartStateView {
        robot_part_id: record.robot_part_id,
        total_owned: record.total_owned,
        unassigned: record.unassigned,
        can_buy: record.can_buy,
        can_sell: record.can_sell,
    }
}

pub(super) fn ore_asset_view(record: robominer_db::UserOreAssetStateRecord) -> OreAssetView {
    OreAssetView {
        ore_id: record.ore_id,
        ore_name: record.ore_name,
        amount: record.amount,
        max_allowed: record.max_allowed,
        depot_max_allowed: record.depot_max_allowed,
    }
}

pub(super) fn empty_part_state(robot_part_id: i64) -> PartStateView {
    PartStateView {
        robot_part_id,
        total_owned: 0,
        unassigned: 0,
        can_buy: false,
        can_sell: false,
    }
}
