use crate::html::escape_html;
use crate::robot_page::RobotPageState;

pub(super) fn render_robot_summary(body: &mut String, robot_count: usize, pending_count: usize) {
    body.push_str(r#"<section class="robot-summary" aria-label="Robot fleet">"#);
    body.push_str(r#"<div class="robot-summary-heading">"#);
    body.push_str(r#"<h1 class="robot-page-title">Robot workshop</h1>"#);
    body.push_str("</div>");
    body.push_str(r#"<ul class="robot-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="robot-summary-item"><span class="robot-summary-label">Robots</span><span class="robot-summary-value">{}</span></li>"#,
        robot_count
    ));
    body.push_str(&format!(
        r#"<li class="robot-summary-item"><span class="robot-summary-label">Pending changes</span><span class="robot-summary-value">{}</span></li>"#,
        pending_count
    ));
    body.push_str("</ul></section>");
}

pub(super) fn render_robot_claim_banner(body: &mut String, state: &RobotPageState) {
    body.push_str(&crate::html::render_claimed_ore_rewards_banner(
        "robot-claim-banner",
        &state.claimed_results,
        true,
    ));
}

pub(super) fn render_robot_message(body: &mut String, state: &RobotPageState) {
    let Some(message) = &state.message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "robot-banner robot-banner-error"
    } else {
        "robot-banner robot-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}

pub(super) fn render_robot_fleet_card(
    body: &mut String,
    robot: &robominer_db::RobotConfigStateRecord,
    program_sources: &[robominer_db::ProgramSourceRecord],
    active: bool,
) {
    let active_class = if active {
        " robot-fleet-card-active"
    } else {
        ""
    };
    let program_size = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .map(|program_source| program_source.compiled_size)
        .unwrap_or(0);
    let status_class = if robot.change_pending {
        "robot-fleet-status-pending"
    } else {
        "robot-fleet-status-ready"
    };
    let status_label = if robot.change_pending {
        "Pending"
    } else {
        "Ready"
    };

    body.push_str(&format!(
        r#"<button type="button" class="robot-fleet-card{active_class}" data-robot-id="{}">"#,
        robot.robot_id
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-heading"><span class="robot-fleet-name">{}</span><span class="robot-fleet-status {status_class}">{status_label}</span></span>"#,
        escape_html(&robot.robot_name)
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-highlights"><span>Ore {}</span><span>Mining {}</span><span>CPU {}</span></span>"#,
        robot.max_ore, robot.mining_speed, robot.cpu_speed
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-memory">Memory {}/{}</span>"#,
        program_size, robot.memory_size
    ));
    body.push_str("</button>");
}
