pub(super) const EDIT_CODE_PAGE_SCRIPT: &str = r#"<script>
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
</script>"#;
