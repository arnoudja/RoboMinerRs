use super::{default_edit_code_program_source, edit_code_program_source_from_state};
use crate::edit_code_page::EditCodePageState;
use crate::html::{escape_html, layout};

use super::editor::render_edit_code_panel;
use super::library::{render_edit_code_new_program_card, render_edit_code_program_card};

#[allow(unused_imports)]
pub(super) use super::editor::{
    edit_code_line_count, render_edit_code_line_numbers, render_edit_code_source_field,
};

pub(super) fn render_edit_code_page(
    username: String,
    hud: Option<&str>,
    state: &EditCodePageState,
) -> String {
    let selected_name = if state.selected_program_source_id > 0 {
        state.selected_program_source.source_name.as_str()
    } else {
        "New program"
    };

    let mut body = String::from(r#"<div class="edit-code-page">"#);
    render_edit_code_summary(
        &mut body,
        state.program_sources.len(),
        selected_name,
        state.selected_program_source.linked_robot_count,
    );
    render_edit_code_claim_banner(&mut body, state);
    render_edit_code_message(&mut body, state);

    let mut program_sources = state.program_sources.clone();
    program_sources.sort_by(|left, right| {
        let left_has_error = !left.source.error_description.is_empty();
        let right_has_error = !right.source.error_description.is_empty();
        right_has_error
            .cmp(&left_has_error)
            .then_with(|| right.linked_robot_count.cmp(&left.linked_robot_count))
            .then_with(|| left.source.source_name.cmp(&right.source.source_name))
    });

    body.push_str(r#"<div class="edit-code-deck">"#);
    body.push_str(
        r#"<section class="edit-code-library" aria-labelledby="edit-code-library-title">"#,
    );
    body.push_str(
        r#"<h2 id="edit-code-library-title" class="edit-code-section-title">Programs</h2>"#,
    );
    body.push_str(r#"<p class="edit-code-library-hint">Select a program to edit source code.</p>"#);
    body.push_str(r#"<div class="edit-code-program-cards">"#);
    for program_source in &program_sources {
        let program = edit_code_program_source_from_state(program_source);
        render_edit_code_program_card(
            &mut body,
            program_source.source.id,
            &program,
            program_source.source.id == state.selected_program_source_id,
        );
    }
    render_edit_code_new_program_card(&mut body, state.selected_program_source_id <= 0);
    body.push_str("</div></section>");
    body.push_str(r#"<div class="edit-code-editor-area">"#);
    body.push_str(r#"<div class="edit-code-panels">"#);
    for program_source in &program_sources {
        let program = edit_code_program_source_from_state(program_source);
        render_edit_code_panel(
            &mut body,
            program_source.source.id,
            &program,
            program_source.source.id == state.selected_program_source_id,
        );
    }
    render_edit_code_panel(
        &mut body,
        -1,
        &default_edit_code_program_source(),
        state.selected_program_source_id <= 0,
    );
    body.push_str("</div></div></div>");

    body.push_str(super::scripts::EDIT_CODE_PAGE_SCRIPT);
    body.push_str("</div>");

    layout("RoboMiner - Edit code", "editCode", &username, hud, &body)
}

fn render_edit_code_summary(
    body: &mut String,
    program_count: usize,
    selected_name: &str,
    linked_robot_count: i64,
) {
    body.push_str(r#"<section class="edit-code-summary" aria-label="Program sources">"#);
    body.push_str(r#"<div class="edit-code-summary-heading">"#);
    body.push_str(r#"<h1 class="edit-code-page-title">Code editor</h1>"#);
    body.push_str("</div>");
    body.push_str(r#"<ul class="edit-code-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="edit-code-summary-item"><span class="edit-code-summary-label">Programs</span><span class="edit-code-summary-value">{}</span></li>"#,
        program_count
    ));
    body.push_str(&format!(
        r#"<li class="edit-code-summary-item"><span class="edit-code-summary-label">Selected</span><span class="edit-code-summary-value" id="editCodeSummarySelected">{}</span></li>"#,
        escape_html(selected_name)
    ));
    body.push_str(&format!(
        r#"<li class="edit-code-summary-item"><span class="edit-code-summary-label">Linked robots</span><span class="edit-code-summary-value" id="editCodeSummaryLinkedRobots">{}</span></li>"#,
        linked_robot_count
    ));
    body.push_str("</ul></section>");
}

fn render_edit_code_claim_banner(body: &mut String, state: &EditCodePageState) {
    body.push_str(&crate::html::render_claimed_ore_rewards_banner(
        "edit-code-claim-banner",
        &state.claimed_results,
        true,
    ));
}

fn render_edit_code_message(body: &mut String, state: &EditCodePageState) {
    let Some(message) = &state.message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "edit-code-banner edit-code-banner-error"
    } else {
        "edit-code-banner edit-code-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}
