use crate::html::{escape_html, layout};
use crate::edit_code_page::{EditCodePageState, EditCodeProgramSource};
use super::{
    default_edit_code_program_source, edit_code_apply_server_block_reason,
    edit_code_program_source_from_state, edit_code_save_block_reason,
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
    body.push_str(r#"<section class="edit-code-library" aria-labelledby="edit-code-library-title">"#);
    body.push_str(r#"<h2 id="edit-code-library-title" class="edit-code-section-title">Programs</h2>"#);
    body.push_str(
        r#"<p class="edit-code-library-hint">Select a program to edit source code.</p>"#,
    );
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

    body.push_str(
        r#"<script>
(function() {
    var allowPageUnload = false;

    function syncEditCodeUrl(sourceId) {
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId));
        }
    }

    function setPanelEnabled(panel, enabled) {
        var fields = panel.querySelectorAll('input, select, textarea, button');
        for (var index = 0; index < fields.length; index += 1) {
            fields[index].disabled = !enabled;
        }
    }

    function panelFormSnapshot(panel) {
        var snapshot = {};
        var fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name) {
                snapshot[field.name] = field.value;
            }
        }
        return JSON.stringify(snapshot);
    }

    function isPanelDirty(panel) {
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return false;
        }
        return panelFormSnapshot(panel) !== baseline;
    }

    function capturePanelBaseline(panel) {
        panel.setAttribute('data-form-baseline', panelFormSnapshot(panel));
    }

    function restorePanelBaseline(panel) {
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return;
        }
        var snapshot = JSON.parse(baseline);
        var fields = panel.querySelectorAll('input[name], select[name], textarea[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && Object.prototype.hasOwnProperty.call(snapshot, field.name)) {
                field.value = snapshot[field.name];
            }
        }
        var sourceInput = panel.querySelector('textarea[name="sourceCode"]');
        if (sourceInput) {
            syncLineNumbersForTextarea(sourceInput);
        }
    }

    function sourceCodeLineCount(value) {
        if (!value) {
            return 1;
        }
        return value.split('\n').length;
    }

    function renderLineNumbers(gutter, lineCount) {
        var lines = [];
        for (var line = 1; line <= lineCount; line += 1) {
            lines.push(String(line));
        }
        gutter.innerHTML = lines.join('<br>');
    }

    function syncLineNumbersForTextarea(textarea) {
        var editor = textarea.closest('.edit-code-source-editor');
        if (!editor) {
            return;
        }
        var gutter = editor.querySelector('.edit-code-line-numbers');
        if (!gutter) {
            return;
        }
        renderLineNumbers(gutter, sourceCodeLineCount(textarea.value));
        gutter.scrollTop = textarea.scrollTop;
    }

    function attachLineNumberListeners(textarea) {
        if (textarea.getAttribute('data-line-numbers') === 'true') {
            syncLineNumbersForTextarea(textarea);
            return;
        }
        textarea.setAttribute('data-line-numbers', 'true');
        var editor = textarea.closest('.edit-code-source-editor');
        var gutter = editor && editor.querySelector('.edit-code-line-numbers');
        textarea.addEventListener('input', function() {
            syncLineNumbersForTextarea(textarea);
        });
        textarea.addEventListener('scroll', function() {
            if (gutter) {
                gutter.scrollTop = textarea.scrollTop;
            }
        });
        syncLineNumbersForTextarea(textarea);
    }

    function updateEditCodeSummary(sourceId) {
        var summary = document.getElementById('editCodeSummarySelected');
        var linkedSummary = document.getElementById('editCodeSummaryLinkedRobots');
        var card = document.querySelector('.edit-code-program-card[data-source-id="' + sourceId + '"]');
        if (summary && card) {
            var cardName = card.querySelector('.edit-code-program-name');
            if (cardName) {
                summary.textContent = cardName.textContent;
            }
        }
        if (linkedSummary && card) {
            linkedSummary.textContent = card.getAttribute('data-linked-robots') || '0';
        }
    }

    function updateEditCodeSummaryFromPanel(panel) {
        if (!panel) {
            return;
        }
        var sourceId = panel.getAttribute('data-source-id');
        var nameInput = panel.querySelector('input[name="sourceName"]');
        var summary = document.getElementById('editCodeSummarySelected');
        if (summary && nameInput) {
            var name = nameInput.value.trim();
            if (sourceId === '-1' && !name) {
                summary.textContent = 'New program';
            } else if (name) {
                summary.textContent = name;
            }
        }
    }

    function selectProgramSource(sourceId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        var cards = document.querySelectorAll('.edit-code-program-card');
        var panels = document.querySelectorAll('.edit-code-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (card.getAttribute('data-source-id') === sourceId) {
                card.classList.add('edit-code-program-card-active');
            } else {
                card.classList.remove('edit-code-program-card-active');
            }
        }
        for (var index = 0; index < panels.length; index += 1) {
            var panel = panels[index];
            var isActive = panel.getAttribute('data-source-id') === sourceId;
            panel.classList.toggle('edit-code-panel-active', isActive);
            panel.hidden = !isActive;
            setPanelEnabled(panel, isActive);
            if (isActive && !panel.getAttribute('data-form-baseline')) {
                capturePanelBaseline(panel);
            }
            if (isActive) {
                attachEditCodeFieldListeners(panel);
                syncEditCodeFormState(panel);
                updateEditCodeSaveState(panel);
                updateEditCodeApplyState(panel);
            }
        }
        updateEditCodeSummary(sourceId);
        if (updateUrl) {
            syncEditCodeUrl(sourceId);
        }
    }

    function syncEditCodeFormState(panel) {
        if (!panel) {
            return;
        }
        var sourceId = panel.getAttribute('data-source-id');
        var nextInput = panel.querySelector('input[name="nextProgramSourceId"]');
        var programInput = panel.querySelector('input[name="programSourceId"]');
        if (nextInput && sourceId) {
            nextInput.value = sourceId;
        }
        if (programInput && sourceId) {
            programInput.value = sourceId;
        }
    }

    function editCodeSaveBlockReason(panel) {
        var nameInput = panel.querySelector('input[name="sourceName"]');
        var sourceInput = panel.querySelector('textarea[name="sourceCode"]');
        if (nameInput && !nameInput.value.trim()) {
            return 'Program name may not be empty.';
        }
        if (sourceInput && !sourceInput.value.trim()) {
            return 'Program source may not be empty.';
        }
        return null;
    }

    function updateEditCodeDirtyState(panel) {
        if (!panel) {
            return;
        }
        var dirty = isPanelDirty(panel);
        var savedBadge = panel.querySelector('.edit-code-status-saved');
        var dirtyBadge = panel.querySelector('.edit-code-status-dirty');
        var resetButton = panel.querySelector('.edit-code-reset-btn');
        if (savedBadge) {
            savedBadge.hidden = dirty;
        }
        if (dirtyBadge) {
            dirtyBadge.hidden = !dirty;
        }
        if (resetButton) {
            resetButton.hidden = !dirty;
        }
    }

    function updateEditCodeSaveState(panel) {
        if (!panel) {
            return;
        }
        var reason = editCodeSaveBlockReason(panel);
        var saveButton = panel.querySelector('.edit-code-btn-primary');
        var hint = panel.querySelector('.edit-code-save-hint');
        if (saveButton) {
            saveButton.disabled = !!reason;
            if (reason) {
                saveButton.setAttribute('title', reason);
            } else {
                saveButton.removeAttribute('title');
            }
        }
        if (hint) {
            if (reason) {
                hint.textContent = reason;
                hint.hidden = false;
            } else {
                hint.textContent = '';
                hint.hidden = true;
            }
        }
        updateEditCodeDirtyState(panel);
    }

    function editCodeApplyBlockReason(panel) {
        if (isPanelDirty(panel)) {
            return 'Save program before updating linked robots.';
        }
        return null;
    }

    function updateEditCodeApplyState(panel) {
        if (!panel) {
            return;
        }
        var applyButton = panel.querySelector('.edit-code-apply-btn');
        if (!applyButton) {
            return;
        }
        var serverBlock = applyButton.getAttribute('data-server-block');
        var reason = serverBlock || editCodeApplyBlockReason(panel);
        var hint = panel.querySelector('.edit-code-apply-hint');
        if (serverBlock) {
            applyButton.disabled = true;
            applyButton.setAttribute('title', serverBlock);
        } else {
            applyButton.disabled = !!reason;
            if (reason) {
                applyButton.setAttribute('title', reason);
            } else {
                applyButton.removeAttribute('title');
            }
        }
        if (hint) {
            if (reason && !serverBlock) {
                hint.textContent = reason;
                hint.hidden = false;
            } else {
                hint.textContent = '';
                hint.hidden = true;
            }
        }
    }

    function attachEditCodeFieldListeners(panel) {
        if (panel.getAttribute('data-field-listeners') === 'true') {
            return;
        }
        panel.setAttribute('data-field-listeners', 'true');
        var nameInput = panel.querySelector('input[name="sourceName"]');
        var sourceInput = panel.querySelector('textarea[name="sourceCode"]');
        if (nameInput) {
            nameInput.addEventListener('input', function() {
                updateEditCodeSaveState(panel);
                updateEditCodeApplyState(panel);
                updateEditCodeSummaryFromPanel(panel);
            });
        }
        if (sourceInput) {
            attachLineNumberListeners(sourceInput);
            sourceInput.addEventListener('input', function() {
                updateEditCodeSaveState(panel);
                updateEditCodeApplyState(panel);
            });
        }
    }

    function editCodeUrlSourceId() {
        var search = window.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === 'nextProgramSourceId' && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
    }

    var preferredSourceId = editCodeUrlSourceId();
    if (preferredSourceId && document.querySelector('.edit-code-panel[data-source-id="' + preferredSourceId + '"]')) {
        selectProgramSource(preferredSourceId, false);
    } else {
        var firstCard = document.querySelector('.edit-code-program-card');
        if (firstCard) {
            selectProgramSource(firstCard.getAttribute('data-source-id'), false);
        }
    }

    var programCards = document.querySelectorAll('.edit-code-program-card');
    for (var programIndex = 0; programIndex < programCards.length; programIndex += 1) {
        programCards[programIndex].addEventListener('click', function(event) {
            var sourceId = event.currentTarget.getAttribute('data-source-id');
            var activePanel = document.querySelector('.edit-code-panel-active');
            if (activePanel
                && activePanel.getAttribute('data-source-id') !== sourceId
                && isPanelDirty(activePanel)) {
                var nameInput = activePanel.querySelector('input[name="sourceName"]');
                var programName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this program';
                robominerConfirm('Discard unsaved changes to ' + programName + '?', function(confirmed) {
                    if (!confirmed) {
                        return;
                    }
                    restorePanelBaseline(activePanel);
                    updateEditCodeSaveState(activePanel);
                    updateEditCodeApplyState(activePanel);
                    updateEditCodeSummaryFromPanel(activePanel);
                    selectProgramSource(sourceId);
                });
                return;
            }
            selectProgramSource(sourceId);
        });
    }

    var resetButtons = document.querySelectorAll('.edit-code-reset-btn');
    for (var resetIndex = 0; resetIndex < resetButtons.length; resetIndex += 1) {
        resetButtons[resetIndex].addEventListener('click', function(event) {
            var panel = event.target.closest('.edit-code-panel');
            if (!panel) {
                return;
            }
            restorePanelBaseline(panel);
            updateEditCodeSaveState(panel);
            updateEditCodeApplyState(panel);
            updateEditCodeSummaryFromPanel(panel);
        });
    }

    window.addEventListener('beforeunload', function(event) {
        if (allowPageUnload) {
            return;
        }
        var panels = document.querySelectorAll('.edit-code-panel');
        for (var unloadIndex = 0; unloadIndex < panels.length; unloadIndex += 1) {
            if (isPanelDirty(panels[unloadIndex])) {
                event.preventDefault();
                event.returnValue = '';
                return;
            }
        }
    });

    function confirmEditCodeApply(event) {
        var panel = event.target.closest('.edit-code-panel');
        if (!panel) {
            return;
        }
        var form = event.target.closest('.edit-code-apply-form');
        if (!form) {
            return;
        }
        var applyButton = panel.querySelector('.edit-code-apply-btn');
        if (applyButton && applyButton.disabled) {
            event.preventDefault();
            return;
        }
        if (form.getAttribute('data-robominer-confirmed') === '1') {
            form.removeAttribute('data-robominer-confirmed');
            return;
        }
        event.preventDefault();
        robominerConfirm('Update linked robots using the saved program?', function(confirmed) {
            if (!confirmed) {
                return;
            }
            allowPageUnload = true;
            var sourceId = panel.getAttribute('data-source-id');
            if (sourceId) {
                form.action = 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId);
            }
            form.setAttribute('data-robominer-confirmed', '1');
            if (typeof form.requestSubmit === 'function') {
                form.requestSubmit(event.submitter || undefined);
            } else {
                form.submit();
            }
        });
    }

    function confirmEditCodeSave(event) {
        var panel = event.target.closest('.edit-code-panel');
        if (!panel) {
            return;
        }
        var form = event.target.closest('.edit-code-save-form');
        if (!form) {
            return;
        }
        if (form.getAttribute('data-robominer-confirmed') === '1') {
            form.removeAttribute('data-robominer-confirmed');
            return;
        }
        var nameInput = panel.querySelector('input[name="sourceName"]');
        var programName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this program';
        event.preventDefault();
        robominerConfirm('Save changes to ' + programName + '?', function(confirmed) {
            if (!confirmed) {
                return;
            }
            allowPageUnload = true;
            var sourceId = panel.getAttribute('data-source-id');
            if (sourceId) {
                form.action = 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId);
            }
            form.setAttribute('data-robominer-confirmed', '1');
            if (typeof form.requestSubmit === 'function') {
                form.requestSubmit(event.submitter || undefined);
            } else {
                form.submit();
            }
        });
    }

    function confirmEditCodeDelete(event) {
        var form = event.target.closest('.edit-code-delete-form');
        if (!form) {
            return;
        }
        if (form.getAttribute('data-robominer-confirmed') === '1') {
            form.removeAttribute('data-robominer-confirmed');
            return;
        }
        event.preventDefault();
        var panel = event.target.closest('.edit-code-panel');
        var programName = 'this program';
        if (panel) {
            var nameInput = panel.querySelector('input[name="sourceName"]');
            if (nameInput && nameInput.value.trim()) {
                programName = nameInput.value.trim();
            }
        }
        robominerConfirm('Delete ' + programName + '? This cannot be undone.', function(confirmed) {
            if (!confirmed) {
                return;
            }
            allowPageUnload = true;
            var sourceId = panel && panel.getAttribute('data-source-id');
            if (sourceId) {
                form.action = 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId);
            }
            form.setAttribute('data-robominer-confirmed', '1');
            if (typeof form.requestSubmit === 'function') {
                form.requestSubmit(event.submitter || undefined);
            } else {
                form.submit();
            }
        });
    }

    var saveForms = document.querySelectorAll('.edit-code-save-form');
    for (var saveIndex = 0; saveIndex < saveForms.length; saveIndex += 1) {
        saveForms[saveIndex].addEventListener('submit', confirmEditCodeSave);
    }

    var applyForms = document.querySelectorAll('.edit-code-apply-form');
    for (var applyIndex = 0; applyIndex < applyForms.length; applyIndex += 1) {
        applyForms[applyIndex].addEventListener('submit', confirmEditCodeApply);
    }

    var deleteForms = document.querySelectorAll('.edit-code-delete-form');
    for (var deleteIndex = 0; deleteIndex < deleteForms.length; deleteIndex += 1) {
        deleteForms[deleteIndex].addEventListener('submit', confirmEditCodeDelete);
    }
})();
</script>"#,
    );
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

fn render_edit_code_program_card(
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

fn render_edit_code_new_program_card(body: &mut String, active: bool) {
    let active_class = if active {
        " edit-code-program-card-active"
    } else {
        ""
    };
    let compiled_size = edit_code_compiled_size_label(default_edit_code_program_source().compiled_size);

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

fn edit_code_program_status(
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

fn edit_code_compiled_size_label(compiled_size: i32) -> String {
    if compiled_size >= 0 {
        compiled_size.to_string()
    } else {
        "unknown".to_string()
    }
}

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

fn render_edit_code_panel(
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

fn render_edit_code_save_action(
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
    html.push_str(r#"<p class="edit-code-save-helper">Save compiles and stores your program source.</p>"#);
    html
}

fn edit_code_save_button(block_reason: Option<&str>, disabled_attr: &str) -> String {
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

fn render_edit_code_apply_action(
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

fn render_edit_code_delete_action(
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

