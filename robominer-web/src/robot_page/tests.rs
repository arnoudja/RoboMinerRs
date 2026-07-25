use std::collections::HashMap;
use std::path::PathBuf;

use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};
use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::render::render_robot_page;
use super::{
    RobotPageState, robot_apply_block_reason, robot_page, update_robot_config_rejection_message,
};

fn sample_robot_state(message: Option<String>) -> RobotPageState {
    RobotPageState {
        selected_robot_id: 7,
        program_sources: vec![robominer_db::ProgramSourceRecord {
            id: 11,
            user_id: 1,
            source_name: "Source <One>".to_string(),
            source_code: Some("move();".to_string()),
            verified: true,
            compiled_size: 12,
            error_description: String::new(),
        }],
        robots: vec![robominer_db::RobotConfigStateRecord {
            robot_id: 7,
            robot_name: "Bot <One>".to_string(),
            program_source_id: 11,
            ore_container_id: 101,
            ore_container_name: "Container & Current".to_string(),
            mining_unit_id: 201,
            mining_unit_name: "Mining Unit".to_string(),
            battery_id: 301,
            battery_name: "Battery".to_string(),
            memory_module_id: 401,
            memory_module_name: "Memory <Current>".to_string(),
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
            forward_speed: 1.234,
            backward_speed: 2.345,
            rotate_speed: 90,
            robot_size: 1.987,
            scan_time: 6,
            scan_distance: 5,
            change_pending: false,
        }],
        part_assets: vec![
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 1,
                robot_part_id: 102,
                part_name: "Container <Spare>".to_string(),
                memory_capacity: 0,
                unassigned: 1,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 1,
                robot_part_id: 103,
                part_name: "Container Hidden".to_string(),
                memory_capacity: 0,
                unassigned: 0,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 4,
                robot_part_id: 401,
                part_name: "Memory <Current>".to_string(),
                memory_capacity: 20,
                unassigned: 0,
            },
            robominer_db::RobotConfigPartAssetStateRecord {
                type_id: 4,
                robot_part_id: 402,
                part_name: "Memory & Spare".to_string(),
                memory_capacity: 30,
                unassigned: 1,
            },
        ],
        message,
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
    }
}

fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn robot_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    let response = robot_page(&authenticated_request("/robot"), &config).await;
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert_html_contains(&body, "ROBOMINER_DATABASE_URL");
}

#[test]
fn robot_rendering_preserves_form_contract_and_escapes_fields() {
    let html = render_robot_page(
        "Player".to_string(),
        None,
        &sample_robot_state(Some(
            "Unable to apply robot changes: Invalid <robot>".to_string(),
        )),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="robot-page""#,
            r#"class="robot-summary""#,
            r#"class="robot-deck""#,
            r#"class="robot-fleet-card robot-fleet-card-active""#,
            r#"class="robot-fleet-hint""#,
            r#"class="robot-quick-link robot-quick-link-edit-program""#,
            r#"href="editCode?nextProgramSourceId=11""#,
            r#"href="helpProgramTips">Programming tips</a>"#,
            r#"href="helpMechanics">Mechanics guide</a>"#,
            r#"class="robot-status-badge robot-status-dirty" hidden>Unsaved changes</span>"#,
            r#"class="robot-btn robot-btn-secondary robot-reset-btn" hidden>Reset changes</button>"#,
            "Apply queues part and program changes for this robot.",
            r#"src="js/common/panel_state.js?v="#,
            r#"src="js/common/url_query.js?v="#,
            r#"src="js/robot/page.js?v="#,
            r#"href="miningQueue?robotId=7""#,
            r#"data-compiled-size="12""#,
            r#"data-memory-capacity="20""#,
            "Memory &amp; Spare",
            r#"<form id="robotForm" action="robot" method="post" class="robot-config-form">"#,
            r#"<input type="hidden" name="robotId" value="7"/>"#,
            "Bot &lt;One&gt;",
            "Source &lt;One&gt;",
            "Container &amp; Current",
            "Container &lt;Spare&gt;",
            r#"id="robotName7" name="robotName7""#,
            r#"name="oreContainerId7""#,
            r#"id="memoryModuleId7" name="memoryModuleId7""#,
            r#"class="robot-progress-value">12/20</span>"#,
            r#"class="robot-btn robot-btn-primary">Apply changes</button>"#,
            ">2 minutes<",
            r#"class="robot-banner robot-banner-error">Unable to apply robot changes: Invalid &lt;robot&gt;</p>"#,
        ],
    );
    for absent in [
        r#"<script src="js/robot.js"></script>"#,
        r#"id="robotId""#,
        r#"<button type="submit">Select</button>"#,
        "Container Hidden",
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn robot_shows_success_banner_after_apply() {
    let html = render_robot_page(
        "Player".to_string(),
        None,
        &sample_robot_state(Some("Robot changes queued".to_string())),
    );

    assert_html_contains(
        &html,
        r#"class="robot-banner robot-banner-success">Robot changes queued</p>"#,
    );
}

#[test]
fn robot_shows_claim_banner_when_results_claimed() {
    let mut state = sample_robot_state(None);
    state.claimed_results = robominer_db::ClaimedUserResults {
        claimed_queues: 2,
        ore_rewards: vec![robominer_db::ClaimedOreRewardRecord {
            ore_id: 2,
            ore_name: "Iron".to_string(),
            reward: 12,
        }],
    };

    let html = render_robot_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"class="robot-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#,
            r#"class="claim-banner-reward-amount">+12</span>"#,
        ],
    );
}

#[test]
fn robot_disables_apply_when_program_exceeds_memory() {
    let mut state = sample_robot_state(None);
    state.program_sources[0].compiled_size = 25;

    let html = render_robot_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"class="robot-progress robot-progress-over""#,
            r#"class="robot-btn robot-btn-primary" disabled"#,
            "Not enough memory available.",
        ],
    );
}

#[test]
fn robot_shows_program_compile_hint_without_blocking_apply() {
    let mut state = sample_robot_state(None);
    state.program_sources[0].error_description = "Compile failed".to_string();

    let html = render_robot_page("Player".to_string(), None, &state);

    assert_contains_all(
        &html,
        &[
            r#"data-has-compile-error="1""#,
            r#"class="robot-program-hint">Selected program has a compile error."#,
            r#"class="robot-btn robot-btn-primary">Apply changes</button>"#,
        ],
    );
    assert_html_not_contains(
        &html,
        "Selected program has a compile error. Fix it in the code editor.",
    );
}

#[test]
fn robot_hides_program_compile_hint_when_program_is_valid() {
    let html = render_robot_page("Player".to_string(), None, &sample_robot_state(None));

    assert_html_contains(
        &html,
        r#"class="robot-program-hint" hidden>Selected program has a compile error."#,
    );
}

#[test]
fn robot_apply_block_reason_matches_server_rejections() {
    let robot = sample_robot_state(None).robots[0].clone();
    let program_sources = sample_robot_state(None).program_sources;

    assert_eq!(robot_apply_block_reason(&robot, &program_sources), None);

    let mut pending_robot = robot.clone();
    pending_robot.change_pending = true;
    assert_eq!(
        robot_apply_block_reason(&pending_robot, &program_sources),
        None
    );

    let mut oversized_program = program_sources.clone();
    oversized_program[0].compiled_size = 25;
    assert_eq!(
        robot_apply_block_reason(&robot, &oversized_program),
        Some("Not enough memory available.")
    );
}

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

#[test]
fn robot_update_rejection_messages_are_user_facing() {
    assert_eq!(
        update_robot_config_rejection_message(
            robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending
        ),
        "Changes are already pending for this robot."
    );
    assert_eq!(
        update_robot_config_rejection_message(
            robominer_db::UpdateRobotConfigRejection::ProgramTooLarge
        ),
        "Not enough memory available."
    );
    assert_eq!(
        update_robot_config_rejection_message(
            robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart
        ),
        "No unassigned robot part is available."
    );
}
