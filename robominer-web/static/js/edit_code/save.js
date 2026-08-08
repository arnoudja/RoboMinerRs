var panelState = window.RoboMinerPanelState;
var allowPageUnload = false;

function setPanelEnabled(panel, enabled) {
    panelState.setPanelEnabled(panel, enabled);
}

function isPanelDirty(panel) {
    return panelState.isPanelDirty(panel);
}

function capturePanelBaseline(panel) {
    panelState.capturePanelBaseline(panel);
}

function restorePanelBaseline(panel) {
    panelState.restorePanelBaseline(panel);
    var sourceInput = panel.querySelector('textarea[name="sourceCode"]');
    if (sourceInput) {
        syncLineNumbersForTextarea(sourceInput);
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
            updateEditCodeSummaryFromPanel(panel);
        });
    }
    if (sourceInput) {
        attachLineNumberListeners(sourceInput);
        sourceInput.addEventListener('input', function() {
            updateEditCodeSaveState(panel);
        });
        sourceInput.addEventListener('keydown', function(event) {
            handleEditCodeTabKey(event, sourceInput);
        });
    }
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
        // Keep selection on an existing program after save. For New program (id -1), omit
        // the query so the server can select the created source instead of reopening draft.
        if (sourceId && sourceId !== '-1') {
            form.action = 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId);
        } else {
            form.action = 'editCode';
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
