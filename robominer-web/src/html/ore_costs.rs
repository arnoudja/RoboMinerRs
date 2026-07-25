use std::collections::HashMap;

use super::format::escape_html;

/// Shared entry-cost affordability check for atlas and mining queue.
pub(crate) fn ore_costs_affordable(
    costs: &[(i64, i32)],
    ore_amount_map: &HashMap<i64, i32>,
) -> bool {
    costs
        .iter()
        .all(|(ore_id, amount)| ore_amount_map.get(ore_id).copied().unwrap_or(0) >= *amount)
}

pub(crate) fn render_ore_entry_costs(
    costs: &[(i64, i32, &str)],
    ore_amount_map: &HashMap<i64, i32>,
    affordable_class: &str,
    unaffordable_class: &str,
) -> String {
    if costs.is_empty() {
        return format!(r#"<span class="{affordable_class}">Free</span>"#);
    }
    costs
        .iter()
        .map(|(ore_id, amount, ore_name)| {
            let have = ore_amount_map.get(ore_id).copied().unwrap_or(0);
            if have >= *amount {
                format!(
                    r#"<span class="{affordable_class}">{} {} ✓</span>"#,
                    amount,
                    escape_html(ore_name)
                )
            } else {
                let need = amount - have;
                format!(
                    r#"<span class="{unaffordable_class}">Need {} more {}.</span>"#,
                    need,
                    escape_html(ore_name)
                )
            }
        })
        .collect::<Vec<_>>()
        .join("<br>")
}
