use crate::html::{assert_html_contains, assert_html_not_contains};

use super::super::render::render_robot_page;
use super::fixtures::sample_robot_state;

#[test]
fn robot_allows_apply_when_change_pending() {
    let mut state = sample_robot_state(None);
    state.robots[0].change_pending = true;

    let html = render_robot_page("Player".to_string(), None, &state);

    for absent in [
        r#"class="robot-btn robot-btn-primary" disabled"#,
        "Changes are already pending for this robot.",
    ] {
        assert_html_not_contains(&html, absent);
    }
    assert_html_contains(
        &html,
        r#"class="robot-status-badge robot-status-pending">Changes pending</span>"#,
    );
}

#[test]
fn robot_fleet_sorts_pending_robots_first() {
    let mut state = sample_robot_state(None);
    state.robots.push(robominer_db::RobotConfigStateRecord {
        robot_id: 8,
        robot_name: "Alpha".to_string(),
        program_source_id: 11,
        ore_container_id: 101,
        ore_container_name: "Container".to_string(),
        mining_unit_id: 201,
        mining_unit_name: "Mining Unit".to_string(),
        battery_id: 301,
        battery_name: "Battery".to_string(),
        memory_module_id: 401,
        memory_module_name: "Memory".to_string(),
        cpu_id: 501,
        cpu_name: "CPU".to_string(),
        engine_id: 601,
        engine_name: "Engine".to_string(),
        ore_scanner_id: 701,
        ore_scanner_name: "Ore Scanner".to_string(),
        recharge_time: 120,
        max_ore: 10,
        mining_speed: 2,
        max_turns: 50,
        memory_size: 20,
        cpu_speed: 3,
        forward_speed: 1.0,
        backward_speed: 1.0,
        rotate_speed: 90,
        robot_size: 1.0,
        scan_time: 6,
        scan_distance: 5,
        change_pending: true,
    });

    let html = render_robot_page("Player".to_string(), None, &state);
    let alpha_pos = html
        .find(r#"class="robot-fleet-card" data-robot-id="8""#)
        .expect("pending robot card should appear");
    let bot_pos = html
        .find(r#"class="robot-fleet-card robot-fleet-card-active" data-robot-id="7""#)
        .or_else(|| html.find(r#"class="robot-fleet-card-active" data-robot-id="7""#))
        .or_else(|| html.find(r#"data-robot-id="7""#))
        .expect("selected robot card should appear");
    assert!(
        alpha_pos < bot_pos,
        "pending robots should appear before ready robots in the fleet list"
    );
}
