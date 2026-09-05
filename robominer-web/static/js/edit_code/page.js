(function() {
    const pageRoot = document.querySelector('.edit-code-page');
    const STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-selection-storage-key') || 'robominer.editCode.selectedProgramSourceId'
        : 'robominer.editCode.selectedProgramSourceId';
    const preferStoredSelection = pageRoot
        && pageRoot.getAttribute('data-prefer-stored-selection') === 'true';

    function panelExists(sourceId) {
        return !!(sourceId
            && document.querySelector('.edit-code-panel[data-source-id="' + sourceId + '"]'));
    }

    function readStoredProgramSourceId() {
        const stored = window.RoboMinerSessionStore.readJson(STORAGE_KEY);
        if (stored == null) {
            return null;
        }
        if (typeof stored === 'number' || typeof stored === 'string') {
            return String(stored);
        }
        if (stored.programSourceId != null) {
            return String(stored.programSourceId);
        }
        return null;
    }

    function writeStoredProgramSourceId(sourceId) {
        if (!sourceId || sourceId === '-1') {
            return;
        }
        window.RoboMinerSessionStore.writeJson(STORAGE_KEY, { programSourceId: sourceId });
    }

    function updateEditCodeSummary(sourceId) {
        const summary = document.getElementById('editCodeSummarySelected');
        const linkedSummary = document.getElementById('editCodeSummaryLinkedRobots');
        const card = document.querySelector('.edit-code-program-card[data-source-id="' + sourceId + '"]');
        if (summary && card) {
            const cardName = card.querySelector('.edit-code-program-name');
            if (cardName) {
                summary.textContent = cardName.textContent;
            }
        }
        if (linkedSummary && card) {
            linkedSummary.textContent = card.getAttribute('data-linked-robots') || '0';
        }
    }

    function syncEditCodeFormState(panel) {
        if (!panel) {
            return;
        }
        const sourceId = panel.getAttribute('data-source-id');
        const nextInput = panel.querySelector('input[name="nextProgramSourceId"]');
        const programInput = panel.querySelector('input[name="programSourceId"]');
        if (nextInput && sourceId) {
            nextInput.value = sourceId;
        }
        if (programInput && sourceId) {
            programInput.value = sourceId;
        }
    }

    function selectProgramSource(sourceId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        const cards = document.querySelectorAll('.edit-code-program-card');
        const panels = document.querySelectorAll('.edit-code-panel');
        for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            const card = cards[cardIndex];
            if (card.getAttribute('data-source-id') === sourceId) {
                card.classList.add('edit-code-program-card-active');
            } else {
                card.classList.remove('edit-code-program-card-active');
            }
        }
        for (let index = 0; index < panels.length; index += 1) {
            const panel = panels[index];
            const isActive = panel.getAttribute('data-source-id') === sourceId;
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
        writeStoredProgramSourceId(sourceId);
        if (updateUrl) {
            syncEditCodeUrl(sourceId);
        }
    }

    const preferredSourceId = editCodeUrlSourceId();
    const preferredLine = editCodeUrlLine();
    const preferredExists = preferredSourceId
        && preferredSourceId !== '-1'
        && panelExists(preferredSourceId);
    const storedSourceId = readStoredProgramSourceId();
    if (preferredExists) {
        selectProgramSource(preferredSourceId, false);
    } else if (preferStoredSelection && panelExists(storedSourceId)) {
        selectProgramSource(storedSourceId, true);
    } else {
        // Honor server-rendered selection (e.g. after creating a program) instead of
        // blindly picking the first card or reopening the New program draft from URL -1.
        const activeCard = document.querySelector('.edit-code-program-card-active');
        const fallbackCard = activeCard || document.querySelector('.edit-code-program-card');
        if (fallbackCard) {
            const fallbackId = fallbackCard.getAttribute('data-source-id');
            const syncUrl = preferredSourceId === '-1' && fallbackId && fallbackId !== '-1';
            selectProgramSource(fallbackId, syncUrl);
        }
    }
    if (preferredLine) {
        const activePanel = document.querySelector('.edit-code-panel-active');
        if (activePanel) {
            focusSourceLine(activePanel, preferredLine);
        }
    }

    const programCards = document.querySelectorAll('.edit-code-program-card');
    for (let programIndex = 0; programIndex < programCards.length; programIndex += 1) {
        programCards[programIndex].addEventListener('click', function(event) {
            const sourceId = event.currentTarget.getAttribute('data-source-id');
            const activePanel = document.querySelector('.edit-code-panel-active');
            if (activePanel
                && activePanel.getAttribute('data-source-id') !== sourceId
                && isPanelDirty(activePanel)) {
                const nameInput = activePanel.querySelector('input[name="sourceName"]');
                const programName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this program';
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

    const resetButtons = document.querySelectorAll('.edit-code-reset-btn');
    for (let resetIndex = 0; resetIndex < resetButtons.length; resetIndex += 1) {
        resetButtons[resetIndex].addEventListener('click', function(event) {
            const panel = event.target.closest('.edit-code-panel');
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
        const panels = document.querySelectorAll('.edit-code-panel');
        for (let unloadIndex = 0; unloadIndex < panels.length; unloadIndex += 1) {
            if (isPanelDirty(panels[unloadIndex])) {
                event.preventDefault();
                event.returnValue = '';
                return;
            }
        }
    });

    const saveForms = document.querySelectorAll('.edit-code-save-form');
    for (let saveIndex = 0; saveIndex < saveForms.length; saveIndex += 1) {
        saveForms[saveIndex].addEventListener('submit', confirmEditCodeSave);
    }

    const deleteForms = document.querySelectorAll('.edit-code-delete-form');
    for (let deleteIndex = 0; deleteIndex < deleteForms.length; deleteIndex += 1) {
        deleteForms[deleteIndex].addEventListener('submit', confirmEditCodeDelete);
    }
})();
