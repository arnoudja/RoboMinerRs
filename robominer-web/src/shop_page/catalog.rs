use std::collections::HashMap;

use super::ShopPageState;
use super::helpers::{
    ShopButtonStyle, add_shop_stat_entry, push_shop_highlight, shop_button, shop_cost_summary,
    shop_total_cost,
};
use super::{ENGINE_PART_TYPE_ID, MEMORY_MODULE_PART_TYPE_ID, ORE_SCANNER_PART_TYPE_ID};
use crate::html::{escape_html, format_period};
use crate::mining_area_atlas::{MiningAreaAtlasLinkTarget, render_mining_area_atlas_ore_link};

pub(super) fn render_shop_part_compact_card(
    body: &mut String,
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    part_number: i32,
    page_state: &ShopPageState,
) {
    let active_class = if part.robot_part_id == page_state.selected_part_id {
        " shop-part-card-active"
    } else {
        ""
    };
    let filter_hidden = if part.type_id == page_state.selected_part_type_id
        && part.tier_id == page_state.selected_tier_id
    {
        ""
    } else {
        " shop-filter-hidden"
    };

    body.push_str(&format!(
        r#"<button type="button" class="shop-part-card shop shop-part-card-compact{active_class}{filter_hidden}" id="robotPartTypeRow{}_{}_{}" data-part-id="{}" data-type-id="{}" data-tier-id="{}" data-can-buy="{}" data-cost-total="{}">"#,
        part.type_id,
        part.tier_id,
        part_number,
        part.robot_part_id,
        part.type_id,
        part.tier_id,
        if state.can_buy { "1" } else { "0" },
        shop_total_cost(costs),
    ));
    body.push_str(&format!(
        r#"<span class="shop-part-heading"><span class="shop-part-name shopPartName">{}</span><span class="shop-part-tier">{} quality</span></span>"#,
        escape_html(&part.part_name),
        escape_html(&part.tier_name)
    ));
    body.push_str(r#"<span class="shop-part-highlights">"#);
    push_shop_highlight(body, "Ore", part.ore_capacity, " units");
    push_shop_highlight(body, "Mining", part.mining_capacity, " o/c");
    push_shop_highlight(body, "Battery", part.battery_capacity, " pc");
    push_shop_highlight(body, "CPU", part.cpu_capacity, " i/c");
    if part.type_id == ORE_SCANNER_PART_TYPE_ID {
        push_shop_highlight(body, "Scan", part.scan_distance, "");
    } else if part.type_id == MEMORY_MODULE_PART_TYPE_ID {
        push_shop_highlight(body, "Memory", part.memory_capacity, "");
    } else if part.type_id == ENGINE_PART_TYPE_ID {
        push_shop_highlight(body, "Forward", part.forward_capacity, "");
    } else if part.scan_time > 0 {
        push_shop_highlight(body, "Scan", part.scan_time, " cyc");
    }
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="shop-part-cost-summary">{}</span>"#,
        escape_html(&shop_cost_summary(costs, ore_amount_map))
    ));
    if state.total_owned > 0 {
        body.push_str(&format!(
            r#"<span class="shop-part-owned-badge">Owned: {}</span>"#,
            state.total_owned
        ));
    }
    body.push_str("</button>");
}

pub(super) fn render_shop_part_detail_panel(
    body: &mut String,
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    page_state: &ShopPageState,
) {
    let active_class = if part.robot_part_id == page_state.selected_part_id {
        " shop-detail-panel-active"
    } else {
        ""
    };
    let filter_hidden = if part.type_id == page_state.selected_part_type_id
        && part.tier_id == page_state.selected_tier_id
    {
        ""
    } else {
        " shop-filter-hidden"
    };

    body.push_str(&format!(
        r#"<div class="shop-detail-panel{active_class}{filter_hidden}" id="shopPartDetails{}" data-part-id="{}" data-type-id="{}" data-tier-id="{}"><header class="shop-detail-header"><div><h3 class="shop-part-name shopPartName">{}</h3><p class="shop-part-tier">{} quality</p></div><div class="shop-part-buy shopFirstRow">{}</div></header>"#,
        part.robot_part_id,
        part.robot_part_id,
        part.type_id,
        part.tier_id,
        escape_html(&part.part_name),
        escape_html(&part.tier_name),
        render_shop_buy_action(part, state, costs, ore_amount_map, page_state),
    ));

    body.push_str(r#"<dl class="shop-part-stats">"#);
    add_shop_stat_entry(body, "Ore capacity:", part.ore_capacity, " units");
    add_shop_stat_entry(body, "Mining capacity:", part.mining_capacity, " o/c");
    add_shop_stat_entry(body, "Battery capacity:", part.battery_capacity, " pc");
    add_shop_stat_entry(body, "Memory size:", part.memory_capacity, "");
    add_shop_stat_entry(body, "CPU speed:", part.cpu_capacity, " i/c");
    if part.forward_capacity > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>Engine power</dt><dd>{} forward, {} backward, {} rotate</dd></div>"#,
            part.forward_capacity, part.backward_capacity, part.rotate_capacity
        ));
    }
    if part.scan_time > 0 {
        add_shop_stat_entry(body, "Scan time:", part.scan_time, " cycles");
        add_shop_stat_entry(body, "Scan distance:", part.scan_distance, "");
    }
    add_shop_stat_entry(body, "Power consumption:", part.power_usage, " p");
    if part.recharge_time > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>Recharge time</dt><dd>{}</dd></div>"#,
            format_period(part.recharge_time)
        ));
    }
    add_shop_stat_entry(body, "Weight:", part.weight, "");
    add_shop_stat_entry(body, "Volume:", part.volume, "");
    body.push_str("</dl>");

    if state.total_owned > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-owned"><p class="shop-part-owned-summary"><span class="important">Owned:</span> {} total, {} unassigned</p><div class="shop-part-owned-action">{}</div></div>"#,
            state.total_owned,
            state.unassigned,
            render_shop_sell_action(part, state, page_state),
        ));
    }

    body.push_str(r#"<div class="shop-part-costs"><p class="shop-part-costs-title">Ore cost</p><ul class="shop-part-cost-list">"#);
    for cost in costs {
        let user_amount = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        body.push_str(&format!(
            r#"<li><span>{}: {}</span><span class="{}">({})</span>{}</li>"#,
            escape_html(&cost.ore_name),
            cost.amount,
            if user_amount >= cost.amount {
                "sufficientbalance"
            } else {
                "insufficientbalance"
            },
            user_amount,
            render_mining_area_atlas_ore_link(
                cost.ore_id,
                &cost.ore_name,
                MiningAreaAtlasLinkTarget::StandalonePage,
                "shop-atlas-link",
            ),
        ));
    }
    body.push_str("</ul></div></div>");
}

pub(super) fn render_shop_buy_action(
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    page_state: &ShopPageState,
) -> String {
    let block_reason = shop_buy_block_reason(state, costs, ore_amount_map);
    let mut html = shop_button(
        "Buy part",
        "buyRobotPartId",
        part.robot_part_id,
        page_state,
        state.can_buy,
        ShopButtonStyle::Primary,
        block_reason.as_deref(),
    );
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="shop-action-hint">{}</p>"#,
            escape_html(&reason)
        ));
        if let Some(shortfall) = costs
            .iter()
            .find(|cost| ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0) < cost.amount)
        {
            html.push_str(&render_mining_area_atlas_ore_link(
                shortfall.ore_id,
                &shortfall.ore_name,
                MiningAreaAtlasLinkTarget::StandalonePage,
                "shop-atlas-link shop-action-atlas-link",
            ));
        }
    }
    html
}

pub(super) fn render_shop_sell_action(
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    page_state: &ShopPageState,
) -> String {
    let block_reason = shop_sell_block_reason(state);
    let mut html = shop_button(
        "Sell unassigned",
        "sellRobotPartId",
        part.robot_part_id,
        page_state,
        state.can_sell,
        ShopButtonStyle::Danger,
        block_reason.as_deref(),
    );
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="shop-action-hint">{}</p>"#,
            escape_html(&reason)
        ));
    }
    html
}

pub(super) fn shop_buy_block_reason(
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> Option<String> {
    if state.can_buy {
        return None;
    }
    for cost in costs {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return Some(format!("Need {} more {}.", need, cost.ore_name));
        }
    }
    if state.total_owned > 0 {
        Some("You already own enough of this part for your robots.".to_string())
    } else {
        Some("You need a robot before buying parts.".to_string())
    }
}

pub(super) fn shop_sell_block_reason(
    state: &robominer_db::ShopRobotPartStateRecord,
) -> Option<String> {
    if state.can_sell {
        return None;
    }
    if state.total_owned > 0 {
        Some("All units are assigned to robots.".to_string())
    } else {
        Some("You do not own this part.".to_string())
    }
}
