use std::collections::HashMap;
use std::path::PathBuf;

use crate::session::format_authenticated_cookie;
use crate::{Request, ServerConfig};

use super::{
    EditCodePageState, EditCodeProgramSource, default_edit_code_program_source,
    edit_code_apply_server_block_reason, edit_code_page, edit_code_save_block_reason,
    format_program_source_apply_message, program_source_write_rejection_message,
    selected_edit_code_source,
};
use super::render::{
    edit_code_line_count, render_edit_code_line_numbers, render_edit_code_page,
    render_edit_code_source_field,
};
use robominer_domain::program_source_apply_warning_message;

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

fn sample_edit_code_state(
    selected_program_source_id: i64,
    selected_program_source: EditCodeProgramSource,
    message: Option<String>,
) -> EditCodePageState {
    EditCodePageState {
        selected_program_source_id,
        selected_program_source,
        program_sources: vec![robominer_db::ProgramSourceStateRecord {
            source: robominer_db::ProgramSourceRecord {
                id: 11,
                user_id: 1,
                source_name: "Source <One>".to_string(),
                source_code: Some("move(1);\n// <mine>\nmine();".to_string()),
                verified: false,
                compiled_size: 12,
                error_description: "Compile <error>".to_string(),
            },
            linked_robot_count: 0,
        }],
        message,
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
    }
}

#[test]
fn edit_code_requires_database_configuration() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
    };

    let response = edit_code_page(&authenticated_request("/editCode"), &config);
    let body = String::from_utf8(response.body).expect("message should be utf-8");

    assert_eq!(response.status, 503);
    assert!(body.contains("ROBOMINER_DATABASE_URL"));
}

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

    assert!(!html.contains(r#"<script src="js/editcode.js"></script>"#));
    assert!(html.contains(r#"class="edit-code-page""#));
    assert!(html.contains(r#"class="edit-code-summary""#));
    assert!(html.contains(r#"id="eraseProgramSourceForm11""#));
    assert!(html.contains(r#"id="editCodeForm11""#));
    assert!(!html.contains(r#"id="changeProgramSourceForm""#));
    assert!(!html.contains(r#"<button type="submit">Open</button>"#));
    assert!(html.contains(r#"class="edit-code-deck""#));
    assert!(html.contains(r#"class="edit-code-program-card edit-code-program-card-active" data-source-id="11""#));
    assert!(html.contains(r#"class="edit-code-program-card" data-source-id="-1""#));
    assert!(html.contains(r#"id="editCodeSummarySelected""#));
    assert!(html.contains(r#"id="editCodeSummaryLinkedRobots""#));
    assert!(html.contains(r#"data-linked-robots="0""#));
    assert!(html.contains(r#"name="nextProgramSourceId" value="11""#));
    assert!(html.contains(r#"name="programSourceId" value="11""#));
    assert!(html.contains(r#"id="sourceName11""#));
    assert!(html.contains(r#"name="sourceName""#));
    assert!(html.contains(r#"class="edit-code-source-editor""#));
    assert!(html.contains(r#"id="sourceCodeLines11""#));
    assert!(html.contains(r#"class="edit-code-line-numbers""#));
    assert!(html.contains("1<br>2<br>3"));
    assert!(html.contains(r#"value="Source &lt;One&gt;""#));
    assert!(html.contains("// &lt;mine&gt;"));
    assert!(html.contains(r#">Delete program</button>"#));
    assert!(html.contains(r#">Save program</button>"#));
    assert!(html.contains(r#"class="edit-code-banner edit-code-banner-compile">Compile &lt;error&gt;</p>"#));
    assert!(html.contains(r#"class="edit-code-banner edit-code-banner-error">Unable to save program: Save &lt;warning&gt;</p>"#));
    assert!(html.contains(r#"class="edit-code-status-badge edit-code-status-dirty" hidden>Unsaved changes</span>"#));
    assert!(html.contains(r#"class="edit-code-btn edit-code-btn-secondary edit-code-reset-btn" hidden>Reset changes</button>"#));
    assert!(html.contains(r#"class="edit-code-save-helper">Save compiles and stores your program source.</p>"#));
    assert!(html.contains(r#"class="edit-code-delete-helper">Delete removes this program from your library.</p>"#));
    assert!(html.contains("Compiled size"));
    assert!(html.contains(">12<"));
    assert!(html.contains("function syncLineNumbersForTextarea(textarea)"));
    assert!(html.contains("function attachLineNumberListeners(textarea)"));
    assert!(html.contains("function selectProgramSource(sourceId, updateUrl)"));
    assert!(html.contains("function isPanelDirty(panel)"));
    assert!(html.contains("function editCodeSaveBlockReason(panel)"));
    assert!(html.contains("function updateEditCodeSaveState(panel)"));
    assert!(html.contains("function updateEditCodeSummary(sourceId)"));
    assert!(html.contains("function syncEditCodeFormState(panel)"));
    assert!(html.contains("addEventListener('beforeunload'"));
    assert!(html.contains("form.action = 'editCode?nextProgramSourceId='"));
    assert!(html.contains("function confirmEditCodeSave(event)"));
    assert!(html.contains("function confirmEditCodeApply(event)"));
    assert!(html.contains("function confirmEditCodeDelete(event)"));
    assert!(html.contains("function updateEditCodeApplyState(panel)"));
    assert!(html.contains(
        "if (form.getAttribute('data-robominer-confirmed') === '1') {\n            form.removeAttribute('data-robominer-confirmed');\n            return;\n        }\n        var nameInput = panel.querySelector('input[name=\"sourceName\"]');"
    ));
    assert!(html.contains(
        "if (form.getAttribute('data-robominer-confirmed') === '1') {\n            form.removeAttribute('data-robominer-confirmed');\n            return;\n        }\n        event.preventDefault();\n        var panel = event.target.closest('.edit-code-panel');"
    ));
    assert!(html.contains(r#"class="edit-code-quick-link" href="robot""#));
    assert!(html.contains(r#"class="edit-code-quick-link" href="helpRobotProgram""#));
    assert!(html.contains(r#"class="edit-code-quick-link" href="helpProgramTips""#));
    assert!(!html.contains("alert("));
    assert!(!html.contains(r#"id="programSourceId" name="nextProgramSourceId""#));
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
            claimed_results: robominer_db::ClaimedUserResults {
                claimed_queues: 2,
                ore_rewards: vec![robominer_db::ClaimedOreRewardRecord {
                    ore_id: 1,
                    ore_name: "Cerbonium".to_string(),
                    reward: 5,
                }],
            },
        },
    );

    assert!(html.contains(r#"class="edit-code-banner edit-code-banner-success">Program saved.</p>"#));
    assert!(html.contains(r#"class="edit-code-claim-banner"><span class="claim-banner-label">Added to wallet:</span>"#));
    assert!(html.contains(r#"class="claim-banner-reward-amount">+5</span>"#));
    assert!(html.contains(r#"href="miningResults">View results</a>"#));
}

#[test]
fn edit_code_default_program_is_rendered_when_no_source_is_selected() {
    let html = render_edit_code_page(
        "Player".to_string(),
        None,
        &EditCodePageState {
            selected_program_source_id: -1,
            selected_program_source: default_edit_code_program_source(),
            program_sources: Vec::new(),
            message: Some("Unable to save program: Save <warning>".to_string()),
            claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        },
    );

    assert!(html.contains(r#"class="edit-code-program-card edit-code-program-card-active" data-source-id="-1""#));
    assert!(html.contains(r#"id="editCodePanel-1""#));
    assert!(html.contains("move(1);"));
    assert!(html.contains("mine();"));
    assert!(html.contains("Save &lt;warning&gt;"));
    assert!(!html.contains("alert("));
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
            claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        },
    );

    assert!(html.contains("Compile failed"));
    assert!(html.contains("Compiled size"));
    assert!(html.contains("unknown"));
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
            claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        },
    );

    assert!(html.contains(r#"class="edit-code-btn edit-code-btn-danger" disabled"#));
    assert!(html.contains("Used by 2 robot(s)."));
    assert!(html.contains(r#"class="edit-code-action-link" href="robot">Open robot workshop</a>"#));
    assert!(!html.contains(r#"id="eraseProgramSourceForm11""#));
}

#[test]
fn edit_code_new_program_selection_does_not_fall_back_to_first_source() {
    let sources = vec![robominer_db::ProgramSourceStateRecord {
        source: robominer_db::ProgramSourceRecord {
            id: 11,
            user_id: 1,
            source_name: "Existing".to_string(),
            source_code: Some("move(1);".to_string()),
            verified: true,
            compiled_size: 4,
            error_description: String::new(),
        },
        linked_robot_count: 0,
    }];

    assert_eq!(
        selected_edit_code_source(&sources, None).map(|state| state.source.id),
        Some(11)
    );
    assert_eq!(
        selected_edit_code_source(&sources, Some(11)).map(|state| state.source.id),
        Some(11)
    );
    assert!(
        selected_edit_code_source(&sources, Some(-1)).is_none(),
        "New program must render the default program, not the first existing source"
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

fn linked_verified_edit_code_state() -> EditCodePageState {
    EditCodePageState {
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
        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
    }
}

#[test]
fn edit_code_shows_update_linked_robots_when_verified_and_linked() {
    let html = render_edit_code_page("Player".to_string(), None, &linked_verified_edit_code_state());

    assert!(html.contains(r#"id="editCodeApplyForm11""#));
    assert!(html.contains(r#"class="edit-code-apply-form""#));
    assert!(html.contains(">Update linked robots</button>"));
    assert!(!html.contains(r#"class="edit-code-btn edit-code-btn-secondary edit-code-apply-btn" disabled"#));
    assert!(html.contains("Idle robots with enough memory are updated immediately"));
    assert!(html.contains(r#"name="requestType" value="applyRobots""#));
}

#[test]
fn edit_code_hides_update_linked_robots_when_no_linked_robots() {
    let mut state = linked_verified_edit_code_state();
    state.selected_program_source.linked_robot_count = 0;
    state.program_sources[0].linked_robot_count = 0;

    let html = render_edit_code_page("Player".to_string(), None, &state);

    assert!(!html.contains(r#"id="editCodeApplyForm11""#));
    assert!(!html.contains(r#"name="requestType" value="applyRobots""#));
}

#[test]
fn edit_code_disables_update_linked_robots_when_compile_error() {
    let mut state = linked_verified_edit_code_state();
    state.selected_program_source.verified = false;
    state.selected_program_source.error_description = "Syntax error".to_string();
    state.program_sources[0].source.verified = false;
    state.program_sources[0].source.error_description = "Syntax error".to_string();

    let html = render_edit_code_page("Player".to_string(), None, &state);

    assert!(html.contains(r#"class="edit-code-btn edit-code-btn-secondary edit-code-apply-btn" disabled"#));
    assert!(html.contains("Save and fix compile errors before updating linked robots."));
}

#[test]
fn format_program_source_apply_message_reports_success_and_warnings() {
    assert_eq!(
        format_program_source_apply_message(&robominer_db::AppliedProgramSource {
            applied_robots: 2,
            warnings: vec![],
        }),
        "Updated 2 robot(s)."
    );
    assert_eq!(
        format_program_source_apply_message(&robominer_db::AppliedProgramSource {
            applied_robots: 1,
            warnings: vec![robominer_db::ProgramSourceApplyWarning {
                robot_name: "BusyBot".to_string(),
                reason: robominer_db::ProgramSourceApplyWarningReason::RobotBusy,
            }],
        }),
        "Updated 1 robot(s). Unable to update BusyBot: The robot is busy."
    );
    assert_eq!(
        format_program_source_apply_message(&robominer_db::AppliedProgramSource {
            applied_robots: 0,
            warnings: vec![robominer_db::ProgramSourceApplyWarning {
                robot_name: "TinyBot".to_string(),
                reason: robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory,
            }],
        }),
        "Unable to update linked robots. Unable to update TinyBot: Not enough memory."
    );
    assert_eq!(
        format_program_source_apply_message(&robominer_db::AppliedProgramSource {
            applied_robots: 0,
            warnings: vec![],
        }),
        "Unable to update robots: program has a compile error."
    );
}

#[test]
fn edit_code_apply_server_block_reason_requires_verified_program() {
    assert_eq!(
        edit_code_apply_server_block_reason(&EditCodeProgramSource {
            source_name: "Broken".to_string(),
            source_code: "bad(".to_string(),
            compiled_size: -1,
            error_description: "Syntax error".to_string(),
            linked_robot_count: 1,
            verified: false,
        }),
        Some("Save and fix compile errors before updating linked robots.")
    );
    assert_eq!(
        edit_code_apply_server_block_reason(&EditCodeProgramSource {
            source_name: "Ready".to_string(),
            source_code: "mine();".to_string(),
            compiled_size: 4,
            error_description: String::new(),
            linked_robot_count: 1,
            verified: true,
        }),
        None
    );
    assert_eq!(
        program_source_apply_warning_message(
            robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory
        ),
        "Not enough memory."
    );
}

#[test]
fn edit_code_line_numbers_match_source_line_count() {
    assert_eq!(edit_code_line_count(""), 1);
    assert_eq!(edit_code_line_count("mine();"), 1);
    assert_eq!(edit_code_line_count("move(1);\nrotate(90);"), 2);
    assert_eq!(
        render_edit_code_line_numbers("a\nb\nc"),
        "1<br>2<br>3"
    );
    assert!(render_edit_code_source_field(7, "mine();", "").contains(
        r#"<div class="edit-code-line-numbers" id="sourceCodeLines7" aria-hidden="true">1</div>"#
    ));
}

#[test]
fn edit_code_rejection_messages_are_user_facing() {
    assert_eq!(
        program_source_write_rejection_message(
            robominer_db::ProgramSourceWriteRejection::SourceInUse
        ),
        "Unable to delete program source because it is used by a robot."
    );
    assert_eq!(
        program_source_write_rejection_message(
            robominer_db::ProgramSourceWriteRejection::EmptySourceName
        ),
        "Program name may not be empty."
    );
    assert_eq!(
        program_source_write_rejection_message(
            robominer_db::ProgramSourceWriteRejection::EmptySourceCode
        ),
        "Program source may not be empty."
    );
}
