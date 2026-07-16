use std::collections::HashMap;

use crate::html::layout;
use crate::robot_page::RobotPageState;

use super::config::render_robot_config_panel;
use super::fleet::{
    render_robot_claim_banner, render_robot_fleet_card, render_robot_message, render_robot_summary,
};

pub(super) fn render_robot_page(
    username: String,
    hud: Option<&str>,
    state: &RobotPageState,
) -> String {
    let mut part_asset_map: HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>> =
        HashMap::new();
    for asset in &state.part_assets {
        part_asset_map.entry(asset.type_id).or_default().push(asset);
    }

    let pending_count = state
        .robots
        .iter()
        .filter(|robot| robot.change_pending)
        .count();

    let mut robots = state.robots.clone();
    robots.sort_by(|left, right| {
        right
            .change_pending
            .cmp(&left.change_pending)
            .then_with(|| left.robot_name.cmp(&right.robot_name))
    });

    let mut body = String::from(r#"<div class="robot-page">"#);
    render_robot_summary(&mut body, state.robots.len(), pending_count);
    render_robot_claim_banner(&mut body, state);
    render_robot_message(&mut body, state);

    if state.robots.is_empty() {
        body.push_str(
            r#"<p class="robot-empty">No robots yet. <a href="shop">Visit the shop</a> to buy parts and build your first robot.</p>"#,
        );
    } else {
        body.push_str(r#"<div class="robot-deck">"#);
        body.push_str(r#"<section class="robot-fleet" aria-labelledby="robot-fleet-title">"#);
        body.push_str(r#"<h2 id="robot-fleet-title" class="robot-section-title">Fleet</h2>"#);
        body.push_str(
            r#"<p class="robot-fleet-hint">Select a robot to configure parts and program source.</p>"#,
        );
        body.push_str(r#"<div class="robot-fleet-cards">"#);
        for robot in &robots {
            render_robot_fleet_card(
                &mut body,
                robot,
                &state.program_sources,
                robot.robot_id == state.selected_robot_id,
            );
        }
        body.push_str("</div></section>");
        body.push_str(r#"<div class="robot-config-area">"#);
        body.push_str(
            r#"<form id="robotForm" action="robot" method="post" class="robot-config-form">"#,
        );
        for robot in &robots {
            render_robot_config_panel(
                &mut body,
                robot,
                robot.robot_id == state.selected_robot_id,
                &state.program_sources,
                &part_asset_map,
            );
        }
        body.push_str("</form></div></div>");
    }

    body.push_str(super::scripts::ROBOT_PAGE_SCRIPT);
    body.push_str("</div>");

    layout("RoboMiner - Robot", "robot", &username, hud, &body)
}
