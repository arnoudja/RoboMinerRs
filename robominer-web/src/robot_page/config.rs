use std::collections::HashMap;

use crate::html::{escape_html, format_period, selected_attr};

use super::robot_apply_block_reason;

pub(super) fn render_robot_part_select(
    body: &mut String,
    label: &str,
    field_prefix: &str,
    robot_id: i64,
    type_id: i64,
    current_part_id: i64,
    current_part_name: &str,
    part_asset_map: &HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>>,
    memory_control: bool,
    disabled_attr: &str,
    current_memory_capacity: Option<i32>,
) {
    let id_attr = if memory_control {
        format!(r#" id="{field_prefix}{robot_id}""#)
    } else {
        String::new()
    };
    let current_capacity_attr = current_memory_capacity
        .map(|capacity| format!(r#" data-memory-capacity="{capacity}""#))
        .unwrap_or_default();
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">{}</span><select{id_attr} name="{}{}" class="tableitem robot-select"{disabled_attr}><option value="{}"{current_capacity_attr} selected="selected">{}</option>"#,
        label,
        field_prefix,
        robot_id,
        current_part_id,
        escape_html(current_part_name)
    ));
    for asset in part_asset_map.get(&type_id).into_iter().flatten() {
        if asset.unassigned > 0 && asset.robot_part_id != current_part_id {
            let capacity_attr = if asset.memory_capacity > 0 {
                format!(r#" data-memory-capacity="{}""#, asset.memory_capacity)
            } else {
                String::new()
            };
            body.push_str(&format!(
                r#"<option value="{}"{capacity_attr}>{}</option>"#,
                asset.robot_part_id,
                escape_html(&asset.part_name)
            ));
        }
    }
    body.push_str("</select></label>");
}

pub(super) fn push_robot_highlight(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<span class="robot-stat-highlight"><span class="robot-stat-highlight-label">{label}</span><span class="robot-stat-highlight-value">{value}{suffix}</span></span>"#,
        ));
    }
}

pub(super) fn add_robot_stat_entry(body: &mut String, label: &str, value: String) {
    body.push_str(&format!(
        r#"<div class="robot-stat"><dt>{label}</dt><dd>{value}</dd></div>"#,
    ));
}

pub(super) fn robot_memory_percent(program_size: i32, memory_size: i32) -> f64 {
    if memory_size <= 0 {
        return 100.0;
    }
    ((program_size as f64 / memory_size as f64) * 100.0).clamp(0.0, 100.0)
}

pub(super) fn render_robot_config_panel(
    body: &mut String,
    robot: &robominer_db::RobotConfigStateRecord,
    active: bool,
    program_sources: &[robominer_db::ProgramSourceRecord],
    part_asset_map: &HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>>,
) {
    let active_class = if active {
        " robot-config-panel-active"
    } else {
        ""
    };
    let hidden_attr = if active { "" } else { " hidden" };
    let disabled_attr = if active { "" } else { " disabled" };
    let program_size = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .map(|program_source| program_source.compiled_size)
        .unwrap_or(0);
    let memory_percent = robot_memory_percent(program_size, robot.memory_size);
    let memory_overflow = program_size > robot.memory_size;
    let selected_program_has_error = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .is_some_and(|program_source| !program_source.error_description.is_empty());
    let program_hint_hidden = if active && selected_program_has_error {
        ""
    } else {
        " hidden"
    };

    let change_pending_attr = if robot.change_pending {
        r#" data-change-pending="true""#
    } else {
        r#" data-change-pending="false""#
    };
    body.push_str(&format!(
        r#"<div class="robot-config-panel{active_class}" id="robotDetails{}" data-robot-id="{}"{change_pending_attr}{hidden_attr}>"#,
        robot.robot_id, robot.robot_id
    ));
    body.push_str(&format!(
        r#"<input type="hidden" name="robotId" value="{}"{disabled_attr}/>"#,
        robot.robot_id
    ));
    body.push_str(r#"<header class="robot-config-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="robot-config-title">{}</h2><p class="robot-config-subtitle">Configure parts and program source</p></div>"#,
        escape_html(&robot.robot_name)
    ));
    if robot.change_pending {
        body.push_str(
            r#"<span class="robot-status-badge robot-status-pending">Changes pending</span>"#,
        );
    } else {
        body.push_str(
            r#"<span class="robot-status-badge robot-status-ready">Ready to apply</span>"#,
        );
        body.push_str(
            r#"<span class="robot-status-badge robot-status-dirty" hidden>Unsaved changes</span>"#,
        );
    }
    body.push_str("</header>");
    body.push_str(&format!(
        r#"<div class="robot-quick-links"><a class="robot-quick-link robot-quick-link-edit-program" href="editCode?nextProgramSourceId={}">Edit program</a><a class="robot-quick-link" href="helpProgramTips">Programming tips</a><a class="robot-quick-link" href="helpMechanics">Mechanics guide</a><a class="robot-quick-link" href="miningQueue?robotId={}">Mining queue</a><a class="robot-quick-link" href="shop">Shop parts</a></div>"#,
        robot.program_source_id, robot.robot_id
    ));

    body.push_str(r#"<section class="robot-config-section"><h3 class="robot-section-title">Identity</h3><div class="robot-field-grid">"#);
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">Name</span><input type="text" id="robotName{}" name="robotName{}" class="robot-text-input" value="{}" maxlength="15" pattern="[A-Za-z0-9_]{{1,15}}" placeholder="1 to 15 characters, only letters and numbers" required{disabled_attr} /></label>"#,
        robot.robot_id,
        robot.robot_id,
        escape_html(&robot.robot_name)
    ));
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">Source code</span><select id="programSourceId{}" name="programSourceId{}" class="tableitem robot-select"{disabled_attr}>"#,
        robot.robot_id, robot.robot_id
    ));
    for program_source in program_sources {
        let compile_error_attr = if program_source.error_description.is_empty() {
            String::new()
        } else {
            r#" data-has-compile-error="1""#.to_string()
        };
        body.push_str(&format!(
            r#"<option value="{}" data-compiled-size="{}"{compile_error_attr}{}>{}</option>"#,
            program_source.id,
            program_source.compiled_size,
            selected_attr(robot.program_source_id == program_source.id),
            escape_html(&program_source.source_name)
        ));
    }
    body.push_str("</select>");
    body.push_str(&format!(
        r#"<p class="robot-program-hint"{program_hint_hidden}>Selected program has a compile error. <a class="robot-program-hint-link" href="editCode?nextProgramSourceId={}">Fix in editor</a></p>"#,
        robot.program_source_id
    ));
    body.push_str("</label></div></section>");

    body.push_str(r#"<section class="robot-config-section"><h3 class="robot-section-title">Parts</h3><div class="robot-field-grid">"#);
    render_robot_part_select(
        body,
        "Ore container",
        "oreContainerId",
        robot.robot_id,
        1,
        robot.ore_container_id,
        &robot.ore_container_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Mining unit",
        "miningUnitId",
        robot.robot_id,
        2,
        robot.mining_unit_id,
        &robot.mining_unit_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Battery",
        "batteryId",
        robot.robot_id,
        3,
        robot.battery_id,
        &robot.battery_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Memory module",
        "memoryModuleId",
        robot.robot_id,
        4,
        robot.memory_module_id,
        &robot.memory_module_name,
        part_asset_map,
        true,
        disabled_attr,
        Some(robot.memory_size),
    );
    render_robot_part_select(
        body,
        "CPU",
        "cpuId",
        robot.robot_id,
        5,
        robot.cpu_id,
        &robot.cpu_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Engine",
        "engineId",
        robot.robot_id,
        6,
        robot.engine_id,
        &robot.engine_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Ore scanner",
        "oreScannerId",
        robot.robot_id,
        7,
        robot.ore_scanner_id,
        &robot.ore_scanner_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    body.push_str("</div></section>");

    body.push_str(
        r#"<section class="robot-config-section"><h3 class="robot-section-title">Performance</h3>"#,
    );
    body.push_str(r#"<div class="robot-stat-highlights">"#);
    push_robot_highlight(body, "Ore cap", robot.max_ore, " units");
    push_robot_highlight(body, "Mining", robot.mining_speed, " u/c");
    push_robot_highlight(body, "CPU", robot.cpu_speed, " i/c");
    push_robot_highlight(body, "Cycles", robot.max_turns, "");
    body.push_str("</div>");
    render_robot_memory_progress(
        body,
        program_size,
        robot.memory_size,
        memory_percent,
        memory_overflow,
    );
    body.push_str(r#"<dl class="robot-stat-grid">"#);
    add_robot_stat_entry(
        body,
        "Forward speed:",
        format!("{:.2} s/c", robot.forward_speed),
    );
    add_robot_stat_entry(
        body,
        "Backward speed:",
        format!("{:.2} s/c", robot.backward_speed),
    );
    add_robot_stat_entry(body, "Rotate speed:", format!("{} d/c", robot.rotate_speed));
    add_robot_stat_entry(body, "Size:", format!("{:.2} s", robot.robot_size));
    add_robot_stat_entry(body, "Scan time:", format!("{} cycles", robot.scan_time));
    add_robot_stat_entry(body, "Scan distance:", format!("{}", robot.scan_distance));
    add_robot_stat_entry(body, "Recharge time:", format_period(robot.recharge_time));
    body.push_str("</dl></section>");

    body.push_str(r#"<div class="robot-apply">"#);
    body.push_str(&render_robot_apply_action(
        robot,
        program_sources,
        disabled_attr,
    ));
    body.push_str("</div></div>");
}

pub(super) fn render_robot_memory_progress(
    body: &mut String,
    program_size: i32,
    memory_size: i32,
    percent: f64,
    overflow: bool,
) {
    let overflow_class = if overflow { " robot-progress-over" } else { "" };
    body.push_str(&format!(r#"<div class="robot-progress{overflow_class}">"#));
    body.push_str(&format!(
        r#"<div class="robot-progress-heading"><span>Memory used</span><span class="robot-progress-value">{}/{}</span></div>"#,
        program_size, memory_size
    ));
    body.push_str(r#"<div class="robot-progress-track" aria-hidden="true">"#);
    body.push_str(&format!(
        r#"<div class="robot-progress-bar" style="width: {percent:.1}%"></div>"#
    ));
    body.push_str("</div></div>");
}

pub(super) fn render_robot_apply_action(
    robot: &robominer_db::RobotConfigStateRecord,
    program_sources: &[robominer_db::ProgramSourceRecord],
    disabled_attr: &str,
) -> String {
    let block_reason = robot_apply_block_reason(robot, program_sources);
    let mut html = String::from(r#"<div class="robot-apply-actions">"#);
    html.push_str(&robot_apply_button(block_reason, disabled_attr));
    html.push_str(r#"<button type="button" class="robot-btn robot-btn-secondary robot-reset-btn" hidden>Reset changes</button></div>"#);
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="robot-action-hint">{}</p>"#,
            escape_html(reason)
        ));
    } else {
        html.push_str(r#"<p class="robot-action-hint" hidden></p>"#);
    }
    html.push_str(
        r#"<p class="robot-apply-helper">Apply queues part and program changes for this robot.</p>"#,
    );
    html
}

pub(super) fn robot_apply_button(block_reason: Option<&str>, disabled_attr: &str) -> String {
    let title_attr = block_reason
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    if block_reason.is_some() || !disabled_attr.is_empty() {
        format!(
            r#"<button type="submit" class="robot-btn robot-btn-primary" disabled{title_attr}{disabled_attr}>Apply changes</button>"#
        )
    } else {
        r#"<button type="submit" class="robot-btn robot-btn-primary">Apply changes</button>"#
            .to_string()
    }
}
