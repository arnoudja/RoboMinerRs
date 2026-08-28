use crate::html::{assert_contains_all, assert_html_contains, assert_html_not_contains};

use super::super::editor::{
    edit_code_line_count, render_edit_code_line_numbers, render_edit_code_source_field,
};
use super::super::render::render_edit_code_page;
use super::super::{EditCodePageState, EditCodeProgramSource, edit_code_save_block_reason};
use super::fixtures::sample_edit_code_state;

#[test]
fn edit_code_rendering_preserves_forms_and_escapes_fields() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &sample_edit_code_state(
            11,
            EditCodeProgramSource {
                source_name: "Source <One>".to_string(),
                source_code: "move(1);\n// <mine>\nmine();".to_string(),
                compiled_size: 12,
                error_description: "Compile <error>".to_string(),
                linked_robot_count: 0,
                verified: false,
            },
            Some("Unable to save program: Save <warning>".to_string()),
        ),
    );

    assert_contains_all(
        &html,
        &[
            r#"class="edit-code-page""#,
            r#"data-prefer-stored-selection="true""#,
            r#"data-selection-storage-key="robominer.editCode.selectedProgramSourceId""#,
            r#"class="edit-code-summary""#,
            r#"id="eraseProgramSourceForm11""#,
            r#"id="editCodeForm11""#,
            r#"class="edit-code-deck""#,
            r#"class="edit-code-program-card edit-code-program-card-active" data-source-id="11""#,
            r#"class="edit-code-program-card" data-source-id="-1""#,
            r#"id="editCodeSummarySelected""#,
            r#"id="editCodeSummaryLinkedRobots""#,
            r#"data-linked-robots="0""#,
            r#"name="nextProgramSourceId" value="11""#,
            r#"name="programSourceId" value="11""#,
            r#"id="sourceName11""#,
            r#"name="sourceName""#,
            r#"class="edit-code-source-editor""#,
            r#"id="sourceCodeLines11""#,
            r#"class="edit-code-line-numbers""#,
            "1<br>2<br>3",
            r#"value="Source &lt;One&gt;""#,
            "// &lt;mine&gt;",
            r#">Delete program</button>"#,
            r#">Save program</button>"#,
            r#"class="edit-code-banner edit-code-banner-compile">Compile &lt;error&gt;</p>"#,
            r#"class="edit-code-banner edit-code-banner-error">Unable to save program: Save &lt;warning&gt;</p>"#,
            r#"class="edit-code-status-badge edit-code-status-dirty" hidden>Unsaved changes</span>"#,
            r#"class="edit-code-btn edit-code-btn-secondary edit-code-reset-btn" hidden>Reset changes</button>"#,
            r#"class="edit-code-save-helper">Save compiles and stores your program. Verified programs are applied to linked robots automatically.</p>"#,
            r#"class="edit-code-delete-helper">Delete removes this program from your library.</p>"#,
            "Compiled size",
            ">12<",
            r#"src="js/common/panel_state.js?v="#,
            r#"src="js/common/url_query.js?v="#,
            r#"src="js/common/session_store.js?v="#,
            r#"src="js/edit_code/page.js?v="#,
            r#"class="edit-code-quick-link" href="robot""#,
            r#"class="edit-code-quick-link" href="helpRobotProgram""#,
            r#"class="edit-code-quick-link" href="helpProgramTips""#,
        ],
    );
    for absent in [
        r#"<script src="js/editcode.js"></script>"#,
        r#"id="changeProgramSourceForm""#,
        r#"<button type="submit">Open</button>"#,
        "alert(",
        r#"id="programSourceId" name="nextProgramSourceId""#,
    ] {
        assert_html_not_contains(&html, absent);
    }
}

#[test]
fn edit_code_shows_success_banner_and_claim_feedback() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: 11,
            selected_program_source: EditCodeProgramSource {
                source_name: "Saved".to_string(),
                source_code: "mine();".to_string(),
                compiled_size: 4,
                error_description: String::new(),
                linked_robot_count: 0,
                verified: true,
            },
            program_sources: Vec::new(),
            message: Some("Program saved.".to_string()),
            pending_claim_count: 0,

            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 2,
                ore_rewards: vec![robominer_db::ClaimedOreRewardRecord {
                    ore_id: 1,
                    ore_name: "Cerbonium".to_string(),
                    reward: 5,
                }],
            },
            prefer_stored_selection: false,
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"class="edit-code-banner edit-code-banner-success">Program saved.</p>"#,
            r#"class="edit-code-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#,
            r#"class="claim-banner-reward-amount">+5</span>"#,
            r#"href="miningResults">View results</a>"#,
        ],
    );
}

#[test]
fn edit_code_default_program_is_rendered_when_no_source_is_selected() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: -1,
            selected_program_source: super::super::default_edit_code_program_source(),
            program_sources: Vec::new(),
            message: Some("Unable to save program: Save <warning>".to_string()),
            pending_claim_count: 0,

            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
            prefer_stored_selection: true,
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"class="edit-code-program-card edit-code-program-card-active" data-source-id="-1""#,
            r#"id="editCodePanel-1""#,
            "move(1);",
            "mine();",
            "Save &lt;warning&gt;",
        ],
    );
    assert_html_not_contains(&html, "alert(");
}

#[test]
fn edit_code_rendering_keeps_compiled_size_line_for_invalid_program() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: 11,
            selected_program_source: EditCodeProgramSource {
                source_name: "Broken".to_string(),
                source_code: "mine(".to_string(),
                compiled_size: -1,
                error_description: "Compile failed".to_string(),
                linked_robot_count: 0,
                verified: false,
            },
            program_sources: vec![robominer_db::ProgramSourceStateRecord {
                source: robominer_db::ProgramSourceRecord {
                    id: 11,
                    user_id: 1,
                    source_name: "Broken".to_string(),
                    source_code: Some("mine(".to_string()),
                    verified: false,
                    compiled_size: -1,
                    error_description: "Compile failed".to_string(),
                },
                linked_robot_count: 0,
            }],
            message: None,
            pending_claim_count: 0,

            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
            prefer_stored_selection: true,
        },
    );

    assert_contains_all(&html, &["Compile failed", "Compiled size", "unknown"]);
}

#[test]
fn edit_code_shows_disabled_delete_when_program_is_linked() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: 11,
            selected_program_source: EditCodeProgramSource {
                source_name: "Linked".to_string(),
                source_code: "mine();".to_string(),
                compiled_size: 4,
                error_description: String::new(),
                linked_robot_count: 2,
                verified: true,
            },
            program_sources: vec![robominer_db::ProgramSourceStateRecord {
                source: robominer_db::ProgramSourceRecord {
                    id: 11,
                    user_id: 1,
                    source_name: "Linked".to_string(),
                    source_code: Some("mine();".to_string()),
                    verified: true,
                    compiled_size: 4,
                    error_description: String::new(),
                },
                linked_robot_count: 2,
            }],
            message: None,
            pending_claim_count: 0,

            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
            prefer_stored_selection: true,
        },
    );

    assert_contains_all(
        &html,
        &[
            r#"class="edit-code-btn edit-code-btn-danger" disabled"#,
            "Used by 2 robot(s).",
            r#"class="edit-code-action-link" href="robot">Open robot workshop</a>"#,
        ],
    );
    assert_html_not_contains(&html, r#"id="eraseProgramSourceForm11""#);
}

#[test]
fn edit_code_omits_manual_update_linked_robots_controls() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: 11,
            selected_program_source: EditCodeProgramSource {
                source_name: "Linked".to_string(),
                source_code: "mine();".to_string(),
                compiled_size: 4,
                error_description: String::new(),
                linked_robot_count: 2,
                verified: true,
            },
            program_sources: vec![robominer_db::ProgramSourceStateRecord {
                source: robominer_db::ProgramSourceRecord {
                    id: 11,
                    user_id: 1,
                    source_name: "Linked".to_string(),
                    source_code: Some("mine();".to_string()),
                    verified: true,
                    compiled_size: 4,
                    error_description: String::new(),
                },
                linked_robot_count: 2,
            }],
            message: None,
            pending_claim_count: 0,

            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 0,
                ore_rewards: vec![],
            },
            prefer_stored_selection: true,
        },
    );

    for absent in [
        r#"id="editCodeApplyForm11""#,
        r#"class="edit-code-apply-form""#,
        ">Update linked robots</button>",
        r#"name="requestType" value="applyRobots""#,
    ] {
        assert_html_not_contains(&html, absent);
    }
    assert_html_contains(
        &html,
        r#"class="edit-code-save-helper">Save compiles and stores your program. Verified programs are applied to linked robots automatically.</p>"#,
    );
}

#[test]
fn format_save_with_optional_apply_message_keeps_plain_save_when_nothing_applied() {
    assert_eq!(
        super::super::format_save_with_optional_apply_message(
            "Program saved.",
            &robominer_db::AppliedProgramSource {
                applied_robots: 0,
                warnings: vec![],
            }
        ),
        "Program saved."
    );
    assert_eq!(
        super::super::format_save_with_optional_apply_message(
            "Program saved.",
            &robominer_db::AppliedProgramSource {
                applied_robots: 1,
                warnings: vec![],
            }
        ),
        "Program saved. Updated 1 robot(s)."
    );
    assert_eq!(
        super::super::format_save_with_optional_apply_message(
            "Program saved.",
            &robominer_db::AppliedProgramSource {
                applied_robots: 1,
                warnings: vec![robominer_db::ProgramSourceApplyWarning {
                    robot_name: "BusyBot".to_string(),
                    reason: robominer_db::ProgramSourceApplyWarningReason::RobotBusy,
                }],
            }
        ),
        "Program saved. Updated 1 robot(s). Unable to update BusyBot: The robot is busy."
    );
}

#[test]
fn edit_code_line_numbers_match_source_line_count() {
    assert_eq!(edit_code_line_count(""), 1);
    assert_eq!(edit_code_line_count("mine();"), 1);
    assert_eq!(edit_code_line_count("move(1);\nrotate(90);"), 2);
    assert_eq!(render_edit_code_line_numbers("a\nb\nc"), "1<br>2<br>3");
    assert_html_contains(
        &render_edit_code_source_field(7, "mine();", ""),
        r#"<div class="edit-code-line-numbers" id="sourceCodeLines7" aria-hidden="true">1</div>"#,
    );
}

#[test]
fn edit_code_save_block_reason_matches_server_rejections() {
    assert_eq!(
        edit_code_save_block_reason("", "mine();"),
        Some("Program name may not be empty.")
    );
    assert_eq!(
        edit_code_save_block_reason("Miner", "   "),
        Some("Program source may not be empty.")
    );
    assert_eq!(edit_code_save_block_reason("Miner", "mine();"), None);
}
