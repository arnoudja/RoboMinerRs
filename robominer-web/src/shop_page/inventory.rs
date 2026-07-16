use std::collections::HashMap;

use super::ShopPageState;
use super::catalog::render_shop_sell_action;
use crate::html::escape_html;

pub(super) fn render_shop_inventory(
    body: &mut String,
    state: &ShopPageState,
    part_state_map: &HashMap<i64, &robominer_db::ShopRobotPartStateRecord>,
) {
    body.push_str(r#"<section class="shop-inventory" aria-labelledby="shop-inventory-title">"#);
    body.push_str(r#"<div class="shop-inventory-header">"#);
    body.push_str(r#"<h2 id="shop-inventory-title" class="shop-section-title">Owned items</h2>"#);
    render_shop_sell_all_action(body, state);
    body.push_str("</div>");
    body.push_str(r#"<table class="shop-inventory-table"><thead><tr><th>Item name</th><th>Quality</th><th>Amount</th><th>Unassigned</th><th></th></tr></thead><tbody>"#);

    let mut owned_rows: Vec<(
        &robominer_db::ShopRobotPartCatalogRecord,
        &robominer_db::ShopRobotPartStateRecord,
    )> = state
        .parts
        .iter()
        .filter_map(|part| {
            part_state_map
                .get(&part.robot_part_id)
                .filter(|part_state| part_state.total_owned > 0)
                .map(|part_state| (part, *part_state))
        })
        .collect();
    owned_rows.sort_by(|(left_part, left_state), (right_part, right_state)| {
        right_state
            .unassigned
            .cmp(&left_state.unassigned)
            .then_with(|| left_part.part_name.cmp(&right_part.part_name))
            .then_with(|| left_part.tier_name.cmp(&right_part.tier_name))
    });

    if owned_rows.is_empty() {
        body.push_str(
            r#"<tr><td colspan="5" class="shop-empty">You do not own any robot parts yet.</td></tr>"#,
        );
    } else {
        for (part, part_state) in owned_rows {
            body.push_str(&format!(
                r#"<tr><td class="shop-inventory-name">{}</td><td>{} quality</td><td>{}</td><td>{}</td><td class="shop-inventory-action">{}</td></tr>"#,
                escape_html(&part.part_name),
                escape_html(&part.tier_name),
                part_state.total_owned,
                part_state.unassigned,
                render_shop_sell_action(part, part_state, state)
            ));
        }
    }

    body.push_str("</tbody></table>");
    body.push_str("</section>");
}

pub(super) fn shop_total_unassigned(part_states: &[robominer_db::ShopRobotPartStateRecord]) -> i32 {
    part_states.iter().map(|state| state.unassigned).sum()
}

pub(super) fn render_shop_sell_all_action(body: &mut String, state: &ShopPageState) {
    let unassigned_total = shop_total_unassigned(&state.part_states);
    let enabled = unassigned_total > 0;
    let disabled_attr = if enabled { "" } else { " disabled" };
    let title_attr = if enabled {
        String::new()
    } else {
        r#" title="No unassigned robot parts to sell.""#.to_string()
    };

    body.push_str(&format!(
        r#"<div class="shop-inventory-actions"><form action="shop" method="post" class="shop-action-form shop-sell-all-form" data-unassigned-count="{unassigned_total}"><input type="hidden" name="sellAllUnassigned" value="1"/><input type="hidden" name="selectedRobotPartTypeId" value="{}"/><input type="hidden" name="selectedTierId" value="{}"/><input type="hidden" name="selectedRobotPartId" value="{}"/><button type="submit" class="shop-btn shop-btn-danger"{disabled_attr}{title_attr}>Sell all unassigned</button></form></div>"#,
        state.selected_part_type_id,
        state.selected_tier_id,
        state.selected_part_id,
    ));
}
