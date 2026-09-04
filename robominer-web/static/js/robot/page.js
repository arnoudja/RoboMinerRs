(function() {
    let allowPageUnload = false;
    const panelState = window.RoboMinerPanelState;
    const PANEL_SKIP_NAMES = ['robotId'];

    function setPanelEnabled(panel, enabled) {
        panelState.setPanelEnabled(panel, enabled);
    }

    function isPanelDirty(panel) {
        return panelState.isPanelDirty(panel, PANEL_SKIP_NAMES);
    }

    function capturePanelBaseline(panel) {
        panelState.capturePanelBaseline(panel, PANEL_SKIP_NAMES);
    }

    function restorePanelBaseline(panel) {
        panelState.restorePanelBaseline(panel, PANEL_SKIP_NAMES);
    }

    function syncRobotUrl(robotId) {
        window.RoboMinerUrlQuery.sync('robot', { robotId: robotId });
    }

    function updateRobotQuickLinks(panel) {
        const programSelect = panel.querySelector('select[name^="programSourceId"]');
        const editLink = panel.querySelector('.robot-quick-link-edit-program');
        if (programSelect && editLink) {
            editLink.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programSelect.value);
        }
    }

    function updateRobotDirtyState(panel) {
        if (!panel) {
            return;
        }
        const dirty = isPanelDirty(panel);
        const readyBadge = panel.querySelector('.robot-status-ready');
        const dirtyBadge = panel.querySelector('.robot-status-dirty');
        const resetButton = panel.querySelector('.robot-reset-btn');
        if (readyBadge) {
            readyBadge.hidden = dirty;
        }
        if (dirtyBadge) {
            dirtyBadge.hidden = !dirty;
        }
        if (resetButton) {
            resetButton.hidden = !dirty;
        }
    }

    function selectRobot(robotId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        const cards = document.querySelectorAll('.robot-fleet-card');
        const panels = document.querySelectorAll('.robot-config-panel');
        for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            const card = cards[cardIndex];
            if (card.getAttribute('data-robot-id') === robotId) {
                card.classList.add('robot-fleet-card-active');
            } else {
                card.classList.remove('robot-fleet-card-active');
            }
        }
        for (let index = 0; index < panels.length; index += 1) {
            const panel = panels[index];
            const isActive = panel.getAttribute('data-robot-id') === robotId;
            panel.classList.toggle('robot-config-panel-active', isActive);
            panel.hidden = !isActive;
            setPanelEnabled(panel, isActive);
            if (isActive) {
                if (!panel.getAttribute('data-form-baseline')) {
                    capturePanelBaseline(panel);
                }
                updateRobotApplyState(panel);
            }
        }
        if (updateUrl) {
            syncRobotUrl(robotId);
        }
    }

    function updateRobotMemoryPreview(panel) {
        if (!panel) {
            return;
        }
        const programSelect = panel.querySelector('select[name^="programSourceId"]');
        const memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (!programSelect || !memorySelect) {
            return;
        }
        const programOption = programSelect.options[programSelect.selectedIndex];
        const memoryOption = memorySelect.options[memorySelect.selectedIndex];
        const programSize = parseInt(programOption.getAttribute('data-compiled-size') || '0', 10);
        let memorySize = parseInt(memoryOption.getAttribute('data-memory-capacity') || '0', 10);
        if (memorySize <= 0) {
            memorySize = 1;
        }
        const percent = Math.min(100, Math.max(0, (programSize / memorySize) * 100));
        const valueElement = panel.querySelector('.robot-progress-value');
        const barElement = panel.querySelector('.robot-progress-meter');
        if (valueElement) {
            valueElement.textContent = programSize + '/' + memorySize;
        }
        const progressElement = panel.querySelector('.robot-progress');
        if (progressElement) {
            progressElement.classList.toggle('robot-progress-over', programSize > memorySize);
        }
        if (barElement) {
            barElement.value = percent.toFixed(1);
        }
    }

    function robotApplyBlockReason(panel) {
        const nameInput = panel.querySelector('input[name^="robotName"]');
        if (nameInput) {
            const robotName = nameInput.value.trim();
            if (!robotName || robotName.length > 15 || !/^[A-Za-z0-9_]+$/.test(robotName)) {
                return 'Invalid robot name.';
            }
        }
        const programSelect = panel.querySelector('select[name^="programSourceId"]');
        const memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (programSelect && memorySelect) {
            const selectedProgram = programSelect.options[programSelect.selectedIndex];
            const selectedMemory = memorySelect.options[memorySelect.selectedIndex];
            const programSize = parseInt(selectedProgram.getAttribute('data-compiled-size') || '0', 10);
            const memorySize = parseInt(selectedMemory.getAttribute('data-memory-capacity') || '0', 10);
            if (memorySize > 0 && programSize > memorySize) {
                return 'Not enough memory available.';
            }
        }
        return null;
    }

    function updateRobotProgramHint(panel) {
        const programSelect = panel.querySelector('select[name^="programSourceId"]');
        const hint = panel.querySelector('.robot-program-hint');
        if (!programSelect || !hint) {
            return;
        }
        const programOption = programSelect.options[programSelect.selectedIndex];
        const hasError = programOption.getAttribute('data-has-compile-error') === '1';
        hint.hidden = !hasError;
        if (hasError) {
            const link = hint.querySelector('a');
            if (link) {
                link.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programOption.value);
            }
        }
    }

    function updateRobotApplyState(panel) {
        if (!panel) {
            return;
        }
        const reason = robotApplyBlockReason(panel);
        const applyButton = panel.querySelector('.robot-btn-primary');
        const hint = panel.querySelector('.robot-action-hint');
        if (applyButton) {
            applyButton.disabled = !!reason;
            if (reason) {
                applyButton.setAttribute('title', reason);
            } else {
                applyButton.removeAttribute('title');
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
        updateRobotProgramHint(panel);
        updateRobotQuickLinks(panel);
        updateRobotMemoryPreview(panel);
        updateRobotDirtyState(panel);
    }

    function attachRobotPreviewListeners(panel) {
        const programSelect = panel.querySelector('select[name^="programSourceId"]');
        const memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        const nameInput = panel.querySelector('input[name^="robotName"]');
        if (programSelect) {
            programSelect.addEventListener('change', function() {
                updateRobotApplyState(panel);
            });
        }
        if (memorySelect) {
            memorySelect.addEventListener('change', function() {
                updateRobotApplyState(panel);
            });
        }
        if (nameInput) {
            nameInput.addEventListener('input', function() {
                updateRobotApplyState(panel);
            });
        }
    }

    const preferredRobotId = window.RoboMinerUrlQuery.getParam('robotId');
    if (preferredRobotId && document.querySelector('.robot-config-panel[data-robot-id="' + preferredRobotId + '"]')) {
        selectRobot(preferredRobotId, false);
    } else {
        const firstCard = document.querySelector('.robot-fleet-card');
        if (firstCard) {
            selectRobot(firstCard.getAttribute('data-robot-id'), false);
        }
    }

    const fleetCards = document.querySelectorAll('.robot-fleet-card');
    for (let fleetIndex = 0; fleetIndex < fleetCards.length; fleetIndex += 1) {
        fleetCards[fleetIndex].addEventListener('click', function(event) {
            const robotId = event.currentTarget.getAttribute('data-robot-id');
            const activePanel = document.querySelector('.robot-config-panel-active');
            if (activePanel
                && activePanel.getAttribute('data-robot-id') !== robotId
                && isPanelDirty(activePanel)) {
                const nameInput = activePanel.querySelector('input[name^="robotName"]');
                const robotName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this robot';
                robominerConfirm('Discard unsaved changes to ' + robotName + '?', function(confirmed) {
                    if (!confirmed) {
                        return;
                    }
                    restorePanelBaseline(activePanel);
                    updateRobotApplyState(activePanel);
                    selectRobot(robotId);
                });
                return;
            }
            selectRobot(robotId);
        });
    }

    const resetButtons = document.querySelectorAll('.robot-reset-btn');
    for (let resetIndex = 0; resetIndex < resetButtons.length; resetIndex += 1) {
        resetButtons[resetIndex].addEventListener('click', function(event) {
            const panel = event.target.closest('.robot-config-panel');
            if (!panel) {
                return;
            }
            restorePanelBaseline(panel);
            updateRobotApplyState(panel);
        });
    }

    window.addEventListener('beforeunload', function(event) {
        if (allowPageUnload) {
            return;
        }
        const panels = document.querySelectorAll('.robot-config-panel');
        for (let unloadIndex = 0; unloadIndex < panels.length; unloadIndex += 1) {
            if (isPanelDirty(panels[unloadIndex])) {
                event.preventDefault();
                event.returnValue = '';
                return;
            }
        }
    });

    const panels = document.querySelectorAll('.robot-config-panel');
    for (let panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
        attachRobotPreviewListeners(panels[panelIndex]);
    }

    function confirmRobotApply(event) {
        let panel = null;
        if (event.submitter) {
            panel = event.submitter.closest('.robot-config-panel');
        }
        if (!panel) {
            panel = document.querySelector('.robot-config-panel-active');
        }
        if (!panel) {
            return;
        }
        const applyButton = panel.querySelector('.robot-btn-primary');
        if (applyButton && applyButton.disabled) {
            event.preventDefault();
            return;
        }
        const nameInput = panel.querySelector('input[name^="robotName"]');
        const robotName = nameInput ? nameInput.value.trim() : 'this robot';
        if (robotForm.getAttribute('data-robominer-confirmed') === '1') {
            robotForm.removeAttribute('data-robominer-confirmed');
            return;
        }
        event.preventDefault();
        robominerConfirm('Apply configuration changes to ' + robotName + '?', function(confirmed) {
            if (!confirmed) {
                return;
            }
            allowPageUnload = true;
            capturePanelBaseline(panel);
            updateRobotApplyState(panel);
            const robotId = panel.getAttribute('data-robot-id');
            if (robotId) {
                robotForm.action = 'robot?robotId=' + encodeURIComponent(robotId);
            }
            robotForm.setAttribute('data-robominer-confirmed', '1');
            if (typeof robotForm.requestSubmit === 'function') {
                robotForm.requestSubmit(event.submitter || undefined);
            } else {
                robotForm.submit();
            }
        });
    }

    const robotForm = document.getElementById('robotForm');
    if (robotForm) {
        robotForm.addEventListener('submit', confirmRobotApply);
    }
})();
