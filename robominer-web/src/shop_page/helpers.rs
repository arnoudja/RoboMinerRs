use std::collections::HashMap;

use super::ShopPageState;
use crate::html::escape_html;

pub(super) fn shop_total_cost(costs: &[&robominer_db::ShopRobotPartCostRecord]) -> i32 {
    costs.iter().map(|cost| cost.amount).sum()
}

pub(super) fn shop_cost_summary(
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> String {
    if costs.is_empty() {
        return "No ore cost".to_string();
    }
    for cost in costs {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return format!("Need {} more {}.", need, cost.ore_name);
        }
    }
    let first = costs[0];
    format!("{} {} — affordable", first.amount, first.ore_name)
}

pub(super) fn push_shop_highlight(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<span class="shop-part-highlight"><span class="shop-part-highlight-label">{label}</span><span class="shop-part-highlight-value">{value}{suffix}</span></span>"#,
        ));
    }
}

pub(super) fn add_shop_stat_entry(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>{label}</dt><dd>{value}{suffix}</dd></div>"#,
        ));
    }
}

pub(super) enum ShopButtonStyle {
    Primary,
    Danger,
}

pub(super) fn shop_button(
    label: &str,
    transaction_field_name: &str,
    robot_part_id: i64,
    page_state: &ShopPageState,
    enabled: bool,
    style: ShopButtonStyle,
    block_reason: Option<&str>,
) -> String {
    let disabled_attr = if enabled { "" } else { " disabled" };
    let title_attr = block_reason
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    let class_name = match style {
        ShopButtonStyle::Primary => "shop-btn shop-btn-primary",
        ShopButtonStyle::Danger => "shop-btn shop-btn-danger",
    };
    format!(
        r#"<form action="shop" method="post" class="shop-action-form"><input type="hidden" name="{}" value="{}"/><input type="hidden" name="selectedRobotPartTypeId" value="{}"/><input type="hidden" name="selectedTierId" value="{}"/><input type="hidden" name="selectedRobotPartId" value="{}"/><button type="submit" class="{class_name}"{disabled_attr}{title_attr}>{}</button></form>"#,
        transaction_field_name,
        robot_part_id,
        page_state.selected_part_type_id,
        page_state.selected_tier_id,
        page_state.selected_part_id,
        label
    )
}
