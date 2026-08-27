use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::render::render_robot_page;
use super::super::robot_apply_block_reason;
use super::fixtures::sample_robot_state;

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
fn robot_update_rejection_messages_are_user_facing() {
    assert_eq!(
        robominer_domain::rejection_messages::update_robot_config_rejection_player_message(
            robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending
        ),
        "Changes are already pending for this robot."
    );
    assert_eq!(
        robominer_domain::rejection_messages::update_robot_config_rejection_player_message(
            robominer_db::UpdateRobotConfigRejection::ProgramTooLarge
        ),
        "Not enough memory available."
    );
    assert_eq!(
        robominer_domain::rejection_messages::update_robot_config_rejection_player_message(
            robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart
        ),
        "No unassigned robot part is available."
    );
}
