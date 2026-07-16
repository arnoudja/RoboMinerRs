use crate::edit_code_page::EditCodeProgramSource;
use crate::html::escape_html;

use super::{edit_code_apply_server_block_reason, edit_code_save_block_reason};

pub(super) fn edit_code_line_count(source_code: &str) -> usize {
    source_code.lines().count().max(1)
}

pub(super) fn render_edit_code_line_numbers(source_code: &str) -> String {
    (1..=edit_code_line_count(source_code))
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("<br>")
}

pub(super) fn render_edit_code_source_field(
    program_source_id: i64,
    source_code: &str,
    disabled_attr: &str,
) -> String {
    format!(
        r#"<label class="edit-code-field edit-code-field-source"><span class="edit-code-field-label">Source code</span><div class="edit-code-source-editor"><div class="edit-code-line-numbers" id="sourceCodeLines{program_source_id}" aria-hidden="true">{}</div><textarea id="sourceCode{program_source_id}" name="sourceCode" class="edit-code-textarea" rows="25" cols="100" required{disabled_attr}>{}</textarea></div></label>"#,
        render_edit_code_line_numbers(source_code),
        escape_html(source_code),
    )
}

pub(super) fn render_edit_code_panel(
    body: &mut String,
    program_source_id: i64,
    program: &EditCodeProgramSource,
    active: bool,
) {
    let active_class = if active {
        " edit-code-panel-active"
    } else {
        ""
    };
    let hidden_attr = if active { "" } else { " hidden" };
    let disabled_attr = if active { "" } else { " disabled" };
    let panel_title = if program_source_id > 0 {
        escape_html(&program.source_name)
    } else {
        "New program".to_string()
    };
    let compiled_size = if program.compiled_size >= 0 {
        program.compiled_size.to_string()
    } else {
        "unknown".to_string()
    };

    body.push_str(&format!(
        r#"<div class="edit-code-panel{active_class}" id="editCodePanel{program_source_id}" data-source-id="{program_source_id}"{hidden_attr}>"#
    ));
    body.push_str(r#"<header class="edit-code-panel-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="edit-code-panel-title">{}</h2><p class="edit-code-panel-subtitle">Edit mining program source</p></div>"#,
        panel_title
    ));
    if program_source_id <= 0 {
        body.push_str(r#"<span class="edit-code-status-badge edit-code-status-saved edit-code-status-new">Draft</span>"#);
    } else if !program.error_description.is_empty() {
        body.push_str(r#"<span class="edit-code-status-badge edit-code-status-saved edit-code-status-error">Compile error</span>"#);
    } else if program.verified {
        body.push_str(r#"<span class="edit-code-status-badge edit-code-status-saved edit-code-status-verified">Verified</span>"#);
    } else {
        body.push_str(r#"<span class="edit-code-status-badge edit-code-status-saved edit-code-status-ready">Ready</span>"#);
    }
    body.push_str(r#"<span class="edit-code-status-badge edit-code-status-dirty" hidden>Unsaved changes</span>"#);
    body.push_str("</header>");
    body.push_str(r#"<div class="edit-code-quick-links"><a class="edit-code-quick-link" href="helpRobotProgram">Language reference</a><a class="edit-code-quick-link" href="helpProgramTips">Programming tips</a><a class="edit-code-quick-link" href="robot">Robot workshop</a></div>"#);

    body.push_str(r#"<ul class="edit-code-meta-list">"#);
    body.push_str(&format!(
        r#"<li><span class="edit-code-meta-label">Compiled size</span><span class="edit-code-meta-value">{}</span></li>"#,
        compiled_size
    ));
    if program_source_id > 0 {
        body.push_str(&format!(
            r#"<li><span class="edit-code-meta-label">Linked robots</span><span class="edit-code-meta-value">{}</span></li>"#,
            program.linked_robot_count
        ));
    }
    body.push_str("</ul>");

    if !program.error_description.is_empty() {
        body.push_str(&format!(
            r#"<p class="edit-code-banner edit-code-banner-compile">{}</p>"#,
            escape_html(&program.error_description)
        ));
    }

    body.push_str(&format!(
        r#"<form id="editCodeForm{program_source_id}" action="editCode" method="post" class="edit-code-save-form"><input type="hidden" name="requestType" value="update"{disabled_attr}/><input type="hidden" name="nextProgramSourceId" value="{program_source_id}"{disabled_attr}/><input type="hidden" name="programSourceId" value="{program_source_id}"{disabled_attr}/><label class="edit-code-field"><span class="edit-code-field-label">Program name</span><input id="sourceName{program_source_id}" type="text" name="sourceName" class="edit-code-text-input" value="{}" size="40" placeholder="Please choose a name for your program" required{disabled_attr} /></label>{}{}"#,
        escape_html(&program.source_name),
        render_edit_code_source_field(program_source_id, &program.source_code, disabled_attr),
        render_edit_code_save_action(&program.source_name, &program.source_code, disabled_attr)
    ));
    body.push_str("</form>");

    if program_source_id > 0 && program.linked_robot_count > 0 {
        body.push_str(&render_edit_code_apply_action(
            program_source_id,
            program,
            disabled_attr,
        ));
    }

    if program_source_id > 0 {
        body.push_str(&render_edit_code_delete_action(
            program_source_id,
            program,
            disabled_attr,
        ));
    }

    body.push_str("</div>");
}

pub(super) fn render_edit_code_save_action(
    source_name: &str,
    source_code: &str,
    disabled_attr: &str,
) -> String {
    let block_reason = edit_code_save_block_reason(source_name, source_code);
    let mut html = String::from(r#"<div class="edit-code-actions">"#);
    html.push_str(&edit_code_save_button(block_reason, disabled_attr));
    html.push_str(r#"<button type="button" class="edit-code-btn edit-code-btn-secondary edit-code-reset-btn" hidden>Reset changes</button></div>"#);
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="edit-code-action-hint edit-code-save-hint">{}</p>"#,
            escape_html(reason)
        ));
    } else {
        html.push_str(r#"<p class="edit-code-action-hint edit-code-save-hint" hidden></p>"#);
    }
    html.push_str(
        r#"<p class="edit-code-save-helper">Save compiles and stores your program source.</p>"#,
    );
    html
}

pub(super) fn edit_code_save_button(block_reason: Option<&str>, disabled_attr: &str) -> String {
    let title_attr = block_reason
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    if block_reason.is_some() || !disabled_attr.is_empty() {
        format!(
            r#"<button type="submit" class="edit-code-btn edit-code-btn-primary" disabled{title_attr}{disabled_attr}>Save program</button>"#
        )
    } else {
        r#"<button type="submit" class="edit-code-btn edit-code-btn-primary">Save program</button>"#
            .to_string()
    }
}

pub(super) fn render_edit_code_apply_action(
    program_source_id: i64,
    program: &EditCodeProgramSource,
    disabled_attr: &str,
) -> String {
    let server_block = edit_code_apply_server_block_reason(program);
    let server_block_attr = server_block
        .map(|reason| format!(r#" data-server-block="{}""#, escape_html(reason)))
        .unwrap_or_default();
    let title_attr = server_block
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    let button_disabled = server_block.is_some() || !disabled_attr.is_empty();

    let mut html = format!(
        r#"<div class="edit-code-apply"><form id="editCodeApplyForm{program_source_id}" action="editCode" method="post" class="edit-code-apply-form"><input type="hidden" name="requestType" value="applyRobots"{disabled_attr}/><input type="hidden" name="nextProgramSourceId" value="{program_source_id}"{disabled_attr}/><input type="hidden" name="programSourceId" value="{program_source_id}"{disabled_attr}/><div class="edit-code-actions">"#
    );
    if button_disabled {
        html.push_str(&format!(
            r#"<button type="submit" class="edit-code-btn edit-code-btn-secondary edit-code-apply-btn" disabled{title_attr}{server_block_attr}{disabled_attr}>Update linked robots</button>"#
        ));
    } else {
        html.push_str(&format!(
            r#"<button type="submit" class="edit-code-btn edit-code-btn-secondary edit-code-apply-btn"{server_block_attr}>Update linked robots</button>"#
        ));
    }
    html.push_str("</div>");
    html.push_str(r#"<p class="edit-code-action-hint edit-code-apply-hint" hidden></p>"#);
    if let Some(reason) = server_block {
        html.push_str(&format!(
            r#"<p class="edit-code-action-hint edit-code-apply-server-hint">{}</p>"#,
            escape_html(reason)
        ));
    }
    html.push_str(
        r#"<p class="edit-code-apply-helper">Idle robots with enough memory are updated immediately. Queued robots receive a pending source update that applies when they finish their current run.</p></form></div>"#,
    );
    html
}

pub(super) fn render_edit_code_delete_action(
    program_source_id: i64,
    program: &EditCodeProgramSource,
    disabled_attr: &str,
) -> String {
    if program.linked_robot_count > 0 {
        return format!(
            r#"<div class="edit-code-delete"><button type="button" class="edit-code-btn edit-code-btn-danger" disabled title="{}">Delete program</button><p class="edit-code-action-hint">Used by {} robot(s). <a class="edit-code-action-link" href="robot">Open robot workshop</a></p></div>"#,
            escape_html("Unable to delete program source because it is used by a robot."),
            program.linked_robot_count
        );
    }
    format!(
        r#"<div class="edit-code-delete"><form id="eraseProgramSourceForm{program_source_id}" action="editCode" method="post" class="edit-code-delete-form"><input type="hidden" name="requestType" value="erase"{disabled_attr}/><input type="hidden" name="programSourceId" value="{program_source_id}"{disabled_attr}/><button type="submit" class="edit-code-btn edit-code-btn-danger"{disabled_attr}>Delete program</button></form><p class="edit-code-delete-helper">Delete removes this program from your library.</p></div>"#
    )
}
