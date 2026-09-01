use crate::{Request, Response, ServerConfig, mutation_form_has, mutation_i64, query_i64};

use robominer_db::part_type_id;

mod actions;
mod catalog;
mod helpers;
mod inventory;
mod render;
mod scripts;
mod view_model;

#[cfg(test)]
mod tests;

use view_model::{
    catalog_part_view, ore_asset_view, ore_view, part_cost_view, part_state_view, part_type_view,
};

pub(super) const ORE_SCANNER_PART_TYPE_ID: i64 = part_type_id::ORE_SCANNER;
pub(super) const MEMORY_MODULE_PART_TYPE_ID: i64 = part_type_id::MEMORY_MODULE;
pub(super) const ENGINE_PART_TYPE_ID: i64 = part_type_id::ENGINE;

#[derive(Debug)]
pub(super) struct ShopPageState {
    pub(super) ores: Vec<OreView>,
    pub(super) part_types: Vec<PartTypeView>,
    pub(super) parts: Vec<CatalogPartView>,
    pub(super) costs: Vec<PartCostView>,
    pub(super) part_states: Vec<PartStateView>,
    pub(super) ore_assets: Vec<OreAssetView>,
    pub(super) selected_part_type_id: i64,
    pub(super) selected_tier_id: i64,
    pub(super) selected_part_id: i64,
    pub(super) message: Option<String>,
}

#[derive(Debug)]
pub(super) struct OreView {
    pub(super) id: i64,
    pub(super) ore_name: String,
}

#[derive(Debug)]
pub(super) struct PartTypeView {
    pub(super) id: i64,
    pub(super) type_name: String,
}

#[derive(Debug)]
pub(super) struct CatalogPartView {
    pub(super) robot_part_id: i64,
    pub(super) type_id: i64,
    pub(super) tier_id: i64,
    pub(super) tier_name: String,
    pub(super) part_name: String,
    pub(super) ore_capacity: i32,
    pub(super) mining_capacity: i32,
    pub(super) battery_capacity: i32,
    pub(super) memory_capacity: i32,
    pub(super) cpu_capacity: i32,
    pub(super) forward_capacity: i32,
    pub(super) backward_capacity: i32,
    pub(super) rotate_capacity: i32,
    pub(super) recharge_time: i32,
    pub(super) scan_time: i32,
    pub(super) scan_distance: i32,
    pub(super) weight: i32,
    pub(super) volume: i32,
    pub(super) power_usage: i32,
}

#[derive(Debug)]
pub(super) struct PartCostView {
    pub(super) robot_part_id: i64,
    pub(super) ore_id: i64,
    pub(super) ore_name: String,
    pub(super) amount: i32,
}

#[derive(Debug)]
pub(super) struct PartStateView {
    pub(super) robot_part_id: i64,
    pub(super) total_owned: i32,
    pub(super) unassigned: i32,
    pub(super) can_buy: bool,
    pub(super) can_sell: bool,
}

#[derive(Debug)]
pub(super) struct OreAssetView {
    pub(super) ore_id: i64,
    pub(super) ore_name: String,
    pub(super) amount: i32,
    pub(super) max_allowed: i32,
    pub(super) depot_max_allowed: i32,
}

pub(super) async fn shop_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    let buy_part_id = mutation_i64(request, "buyRobotPartId");
    let sell_part_id = mutation_i64(request, "sellRobotPartId");
    let selected_part_type_id = query_i64(request, "selectedRobotPartTypeId");
    let selected_tier_id = query_i64(request, "selectedTierId");
    let selected_part_id = query_i64(request, "selectedRobotPartId");

    let result = load_shop_state(
        session.pool,
        session.user_id,
        buy_part_id,
        sell_part_id,
        mutation_form_has(request, "sellAllUnassigned"),
        selected_part_type_id,
        selected_tier_id,
        selected_part_id,
    )
    .await;

    match result {
        Ok(state) => {
            session
                .html_with_hud(request, config, |username, hud| {
                    render::render_shop_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("shop", error),
    }
}

#[allow(clippy::too_many_arguments)]
async fn load_shop_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    buy_part_id: Option<i64>,
    sell_part_id: Option<i64>,
    sell_all_unassigned: bool,
    selected_part_type_id: Option<i64>,
    selected_tier_id: Option<i64>,
    selected_part_id: Option<i64>,
) -> Result<ShopPageState, crate::page_context::PageLoadError> {
    let message = actions::apply_shop_mutations(
        pool,
        user_id,
        buy_part_id,
        sell_part_id,
        sell_all_unassigned,
    )
    .await?;

    let ores: Vec<OreView> = robominer_db::list_mining_area_overview_ores_for_user(pool, user_id)
        .await?
        .into_iter()
        .map(|ore| ore_view(ore.ore_id, ore.ore_name))
        .collect();
    let part_types = robominer_db::list_robot_part_types(pool)
        .await?
        .into_iter()
        .map(part_type_view)
        .collect::<Vec<_>>();
    let parts = robominer_db::list_shop_robot_part_catalog(pool)
        .await?
        .into_iter()
        .map(catalog_part_view)
        .collect::<Vec<_>>();
    let selected_part_type_id = selected_part_type_id
        .or_else(|| part_types.first().map(|part_type| part_type.id))
        .unwrap_or(0);
    let selected_tier_id = selected_tier_id
        .filter(|tier_id| ores.iter().any(|ore| ore.id == *tier_id))
        .or_else(|| default_shop_tier_id(&ores))
        .unwrap_or(0);
    let selected_part_id = resolve_selected_part_id(
        selected_part_id,
        &parts,
        selected_part_type_id,
        selected_tier_id,
    );

    Ok(ShopPageState {
        ores,
        part_types,
        parts,
        costs: robominer_db::list_shop_robot_part_costs(pool)
            .await?
            .into_iter()
            .map(part_cost_view)
            .collect(),
        part_states: robominer_db::list_shop_robot_part_states(pool, user_id)
            .await?
            .into_iter()
            .map(part_state_view)
            .collect(),
        ore_assets: robominer_db::list_user_ore_asset_states(pool, user_id)
            .await?
            .into_iter()
            .map(ore_asset_view)
            .collect(),
        selected_part_type_id,
        selected_tier_id,
        selected_part_id,
        message,
    })
}

fn default_shop_tier_id(ores: &[OreView]) -> Option<i64> {
    ores.iter().map(|ore| ore.id).max()
}

fn resolve_selected_part_id(
    selected_part_id: Option<i64>,
    parts: &[CatalogPartView],
    selected_part_type_id: i64,
    selected_tier_id: i64,
) -> i64 {
    if let Some(selected_part_id) = selected_part_id
        && parts
            .iter()
            .any(|part| part.robot_part_id == selected_part_id)
    {
        return selected_part_id;
    }

    parts
        .iter()
        .find(|part| part.type_id == selected_part_type_id && part.tier_id == selected_tier_id)
        .map(|part| part.robot_part_id)
        .or_else(|| parts.first().map(|part| part.robot_part_id))
        .unwrap_or(0)
}
