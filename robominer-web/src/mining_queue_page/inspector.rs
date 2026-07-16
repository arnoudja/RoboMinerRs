use std::collections::HashMap;

use crate::html::{escape_html, format_period};
use crate::mining_queue_page::MiningQueuePageState;

pub(super) fn render_mining_queue_selection_state_inputs(
    body: &mut String,
    state: &MiningQueuePageState,
    current_robot_id: Option<i64>,
) {
    body.push_str(&format!(
        r#"<input type="hidden" name="infoMiningAreaId" value="{}"/>"#,
        state.selected_info_area_id
    ));
    for robot in &state.robots {
        if Some(robot.robot_id) == current_robot_id {
            continue;
        }
        if let Some(selected_area_id) = state.selected_robot_area_ids.get(&robot.robot_id) {
            body.push_str(&format!(
                r#"<input type="hidden" name="miningArea{}" value="{}"/>"#,
                robot.robot_id, selected_area_id
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_mining_area_details(
    body: &mut String,
    area: &robominer_db::MiningQueuePageAreaRecord,
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    supplies: &[&robominer_db::MiningQueuePageAreaSupplyRecord],
    yields: &[&robominer_db::MiningQueuePageAreaYieldRecord],
    robots: &[robominer_db::MiningQueuePageRobotRecord],
    score_map: &HashMap<(i64, i64), f64>,
    ore_amount_map: &HashMap<i64, i32>,
    active: bool,
) {
    let panel_class = if active {
        "mining-queue-area-panel mining-queue-area-panel-active"
    } else {
        "mining-queue-area-panel"
    };
    body.push_str(&format!(
        r#"<tbody id="miningAreaDetails{}" class="{panel_class}">"#,
        area.mining_area_id
    ));
    if !costs.is_empty() {
        body.push_str(r#"<tr><td colspan="4">Upfront costs:</td></tr>"#);
        for cost in costs {
            let user_amount = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
            body.push_str(&format!(
                r#"<tr><td></td><td>{}:</td><td>{}</td><td class="{}">({})</td></tr>"#,
                escape_html(&cost.ore_name),
                cost.amount,
                if user_amount >= cost.amount {
                    "sufficientbalance"
                } else {
                    "insufficientbalance"
                },
                user_amount
            ));
        }
    }
    body.push_str(&format!(
        r#"<tr><td>Tax rate:</td><td colspan="3">{}%</td></tr>"#,
        area.tax_rate
    ));
    body.push_str(&format!(
        r#"<tr><td>Mining time:</td><td colspan="3">{}</td></tr>
<tr><td>Mining cycles:</td><td colspan="3">{}</td></tr>
<tr><td>Area size:</td><td colspan="3">{} x {}</td></tr>
<tr><td colspan="4">Available ore:</td></tr>"#,
        format_period(area.mining_time),
        area.max_moves,
        area.size_x,
        area.size_y
    ));
    for supply in supplies {
        body.push_str(&format!(
            r#"<tr><td></td><td>{}:</td><td colspan="2">h {} / r {}</td></tr>"#,
            escape_html(&supply.ore_name),
            supply.supply,
            supply.radius
        ));
    }
    let mut title_added = false;
    for robot in robots {
        if let Some(score) = score_map.get(&(robot.robot_id, area.mining_area_id))
            && *score > 0.0
        {
            if !title_added {
                body.push_str(r#"<tr><td colspan="4">Robot score:</td></tr>"#);
                title_added = true;
            }
            body.push_str(&format!(
                r#"<tr><td></td><td>{}</td><td colspan="2">{:.1}</td></tr>"#,
                escape_html(&robot.robot_name),
                score
            ));
        }
    }
    body.push_str(r#"<tr><td colspan="4">Historic yield:</td></tr>"#);
    let mut total_percentage = 0.0;
    for area_yield in yields {
        total_percentage += area_yield.percentage;
        body.push_str(&format!(
            r#"<tr><td></td><td>{}:</td><td colspan="2">{:.1}%</td></tr>"#,
            escape_html(&area_yield.ore_name),
            area_yield.percentage
        ));
    }
    body.push_str(&format!(
        r#"<tr><td></td><td>Total:</td><td colspan="2">{total_percentage:.1}%</td></tr></tbody>"#
    ));
}
