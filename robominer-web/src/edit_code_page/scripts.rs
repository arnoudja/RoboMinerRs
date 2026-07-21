pub(super) const EDIT_CODE_PAGE_SCRIPT: &str = r#"<script>
(function() {
    var allowPageUnload = false;

    function syncEditCodeUrl(sourceId) {
        if (!(window.history && window.history.replaceState)) {
            return;
        }
        var url = 'editCode?nextProgramSourceId=' + encodeURIComponent(sourceId);
        var line = editCodeUrlParam('line');
        if (line) {
            url += '&line=' + encodeURIComponent(line);
        }
        window.history.replaceState(null, '', url);
    }

    function editCodeUrlParam(name) {
        var search = window.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === name && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
    }

    function focusSourceLine(panel, lineNumber) {
        var textarea = panel && panel.querySelector('textarea[name="sourceCode"]');
        if (!textarea || typeof lineNumber !== 'number' || isNaN(lineNumber) || lineNumber < 1) {
            return;
        }
        var lines = textarea.value.split('\n');
        if (lines.length === 0) {
            return;
        }
        var targetLine = Math.min(Math.floor(lineNumber), lines.length);
        var start = 0;
        for (var index = 0; index < targetLine - 1; index += 1) {
            start += lines[index].length + 1;
        }
        var end = start + lines[targetLine - 1].length;
        textarea.focus();
        if (typeof textarea.setSelectionRange === 'function') {
            textarea.setSelectionRange(start, end);
        }
        var style = window.getComputedStyle(textarea);
        var lineHeight = parseFloat(style.lineHeight);
        if (!lineHeight || isNaN(lineHeight)) {
            var fontSize = parseFloat(style.fontSize);
            lineHeight = (fontSize && !isNaN(fontSize) ? fontSize : 14) * 1.4;
        }
        var paddingTop = parseFloat(style.paddingTop);
        if (!paddingTop || isNaN(paddingTop)) {
            paddingTop = 0;
        }
        textarea.scrollTop = Math.max(0, paddingTop + (targetLine - 1) * lineHeight - textarea.clientHeight / 3);
        syncLineNumbersForTextarea(textarea);
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

    var EDIT_CODE_INDENT = '    ';

    function emitEditCodeInput(textarea) {
        if (typeof InputEvent === 'function') {
            textarea.dispatchEvent(new InputEvent('input', { bubbles: true }));
        } else {
            var event = document.createEvent('Event');
            event.initEvent('input', true, true);
            textarea.dispatchEvent(event);
        }
    }

    function lineStartIndex(value, index) {
        var start = value.lastIndexOf('\n', Math.max(0, index - 1));
        return start < 0 ? 0 : start + 1;
    }

    function lineEndIndex(value, index) {
        var end = value.indexOf('\n', index);
        return end < 0 ? value.length : end;
    }

    function outdentLine(line) {
        if (line.charAt(0) === '\t') {
            return line.substring(1);
        }
        var remove = 0;
        while (remove < EDIT_CODE_INDENT.length && line.charAt(remove) === ' ') {
            remove += 1;
        }
        return remove > 0 ? line.substring(remove) : line;
    }

    function adjustSelectedLines(textarea, indent) {
        var value = textarea.value;
        var selectionStart = textarea.selectionStart;
        var selectionEnd = textarea.selectionEnd;
        var rangeStart = lineStartIndex(value, selectionStart);
        var rangeEnd = selectionEnd > selectionStart
            ? lineEndIndex(value, Math.max(selectionStart, selectionEnd - 1))
            : lineEndIndex(value, selectionStart);
        var block = value.substring(rangeStart, rangeEnd);
        var lines = block.split('\n');
        var nextLines = [];
        var lineDeltas = [];
        var totalDelta = 0;
        for (var index = 0; index < lines.length; index += 1) {
            var line = lines[index];
            var nextLine = indent ? EDIT_CODE_INDENT + line : outdentLine(line);
            var delta = nextLine.length - line.length;
            lineDeltas.push(delta);
            totalDelta += delta;
            nextLines.push(nextLine);
        }
        if (totalDelta === 0) {
            return;
        }
        var nextBlock = nextLines.join('\n');
        textarea.value = value.substring(0, rangeStart) + nextBlock + value.substring(rangeEnd);

        function mapOffset(offset) {
            var relative = offset - rangeStart;
            if (relative <= 0) {
                return offset;
            }
            var pos = 0;
            var deltaBefore = 0;
            for (var lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
                var lineLength = lines[lineIndex].length;
                var lineEndRel = pos + lineLength;
                if (relative <= lineEndRel || lineIndex === lines.length - 1) {
                    var offsetInLine = relative - pos;
                    var lineDelta = lineDeltas[lineIndex];
                    if (lineDelta < 0) {
                        var removed = -lineDelta;
                        if (offsetInLine <= removed) {
                            return rangeStart + pos + deltaBefore;
                        }
                        return rangeStart + pos + deltaBefore + offsetInLine + lineDelta;
                    }
                    return rangeStart + pos + deltaBefore + offsetInLine + lineDelta;
                }
                pos = lineEndRel + 1;
                deltaBefore += lineDeltas[lineIndex];
            }
            return offset + totalDelta;
        }

        if (typeof textarea.setSelectionRange === 'function') {
            textarea.setSelectionRange(mapOffset(selectionStart), mapOffset(selectionEnd));
        }
        emitEditCodeInput(textarea);
    }

    function insertEditCodeIndent(textarea) {
        var value = textarea.value;
        var selectionStart = textarea.selectionStart;
        var selectionEnd = textarea.selectionEnd;
        textarea.value = value.substring(0, selectionStart)
            + EDIT_CODE_INDENT
            + value.substring(selectionEnd);
        var cursor = selectionStart + EDIT_CODE_INDENT.length;
        if (typeof textarea.setSelectionRange === 'function') {
            textarea.setSelectionRange(cursor, cursor);
        }
        emitEditCodeInput(textarea);
    }

    function handleEditCodeTabKey(event, textarea) {
        if (event.key !== 'Tab' && event.keyCode !== 9) {
            return;
        }
        event.preventDefault();
        var selectionStart = textarea.selectionStart;
        var selectionEnd = textarea.selectionEnd;
        var selected = textarea.value.substring(selectionStart, selectionEnd);
        if (event.shiftKey) {
            adjustSelectedLines(textarea, false);
            return;
        }
        if (selectionStart !== selectionEnd && selected.indexOf('\n') >= 0) {
            adjustSelectedLines(textarea, true);
            return;
        }
        insertEditCodeIndent(textarea);
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

    function editCodeUrlSourceId() {
        return editCodeUrlParam('nextProgramSourceId');
    }

    function editCodeUrlLine() {
        var raw = editCodeUrlParam('line');
        if (!raw) {
            return null;
        }
        var line = parseInt(raw, 10);
        if (isNaN(line) || line < 1) {
            return null;
        }
        return line;
    }

    var preferredSourceId = editCodeUrlSourceId();
    var preferredLine = editCodeUrlLine();
    if (preferredSourceId && document.querySelector('.edit-code-panel[data-source-id="' + preferredSourceId + '"]')) {
        selectProgramSource(preferredSourceId, false);
    } else {
        var firstCard = document.querySelector('.edit-code-program-card');
        if (firstCard) {
            selectProgramSource(firstCard.getAttribute('data-source-id'), false);
        }
    }
    if (preferredLine) {
        var activePanel = document.querySelector('.edit-code-panel-active');
        if (activePanel) {
            focusSourceLine(activePanel, preferredLine);
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

    var deleteForms = document.querySelectorAll('.edit-code-delete-form');
    for (var deleteIndex = 0; deleteIndex < deleteForms.length; deleteIndex += 1) {
        deleteForms[deleteIndex].addEventListener('submit', confirmEditCodeDelete);
    }
})();
</script>"#;
