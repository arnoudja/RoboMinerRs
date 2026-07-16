use crate::edit_code_page::EditCodeProgramSource;
use crate::html::escape_html;

use super::default_edit_code_program_source;

pub(super) fn render_edit_code_program_card(
    body: &mut String,
    program_source_id: i64,
    program: &EditCodeProgramSource,
    active: bool,
) {
    let active_class = if active {
        " edit-code-program-card-active"
    } else {
        ""
    };
    let (status_class, status_label) = edit_code_program_status(program, program_source_id <= 0);
    let compiled_size = edit_code_compiled_size_label(program.compiled_size);

    body.push_str(&format!(
        r#"<button type="button" class="edit-code-program-card{active_class}" data-source-id="{program_source_id}" data-linked-robots="{}">"#,
        program.linked_robot_count
    ));
    body.push_str(&format!(
        r#"<span class="edit-code-program-heading"><span class="edit-code-program-name">{}</span><span class="edit-code-program-status {status_class}">{status_label}</span></span>"#,
        escape_html(&program.source_name)
    ));
    body.push_str(&format!(
        r#"<span class="edit-code-program-highlights"><span>Size {}</span><span>Robots {}</span></span>"#,
        compiled_size, program.linked_robot_count
    ));
    body.push_str("</button>");
}

pub(super) fn render_edit_code_new_program_card(body: &mut String, active: bool) {
    let active_class = if active {
        " edit-code-program-card-active"
    } else {
        ""
    };
    let compiled_size =
        edit_code_compiled_size_label(default_edit_code_program_source().compiled_size);

    body.push_str(&format!(
        r#"<button type="button" class="edit-code-program-card{active_class}" data-source-id="-1" data-linked-robots="0">"#
    ));
    body.push_str(r#"<span class="edit-code-program-heading"><span class="edit-code-program-name">New program</span><span class="edit-code-program-status edit-code-status-new">Draft</span></span>"#);
    body.push_str(&format!(
        r#"<span class="edit-code-program-highlights"><span>Size {}</span><span>Robots 0</span></span>"#,
        compiled_size
    ));
    body.push_str("</button>");
}

pub(super) fn edit_code_program_status(
    program: &EditCodeProgramSource,
    is_new: bool,
) -> (&'static str, &'static str) {
    if is_new {
        return ("edit-code-status-new", "Draft");
    }
    if !program.error_description.is_empty() {
        return ("edit-code-status-error", "Compile error");
    }
    if program.verified {
        return ("edit-code-status-verified", "Verified");
    }
    ("edit-code-status-ready", "Ready")
}

pub(super) fn edit_code_compiled_size_label(compiled_size: i32) -> String {
    if compiled_size >= 0 {
        compiled_size.to_string()
    } else {
        "unknown".to_string()
    }
}
