pub(super) const ROBOT_PAGE_SCRIPT: &str = r#"<script>
(function() {
    var allowPageUnload = false;

    function syncRobotUrl(robotId) {
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', 'robot?robotId=' + encodeURIComponent(robotId));
        }
    }

    function setPanelEnabled(panel, enabled) {
        var fields = panel.querySelectorAll('input, select, button');
        for (var index = 0; index < fields.length; index += 1) {
            fields[index].disabled = !enabled;
        }
    }

    function panelFormSnapshot(panel) {
        var snapshot = {};
        var fields = panel.querySelectorAll('input[name], select[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && field.name !== 'robotId') {
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
        var fields = panel.querySelectorAll('input[name], select[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && field.name !== 'robotId' && Object.prototype.hasOwnProperty.call(snapshot, field.name)) {
                field.value = snapshot[field.name];
            }
        }
    }

    function updateRobotQuickLinks(panel) {
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var editLink = panel.querySelector('.robot-quick-link-edit-program');
        if (programSelect && editLink) {
            editLink.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programSelect.value);
        }
    }

    function updateRobotDirtyState(panel) {
        if (!panel) {
            return;
        }
        var dirty = isPanelDirty(panel);
        var readyBadge = panel.querySelector('.robot-status-ready');
        var dirtyBadge = panel.querySelector('.robot-status-dirty');
        var resetButton = panel.querySelector('.robot-reset-btn');
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
        var cards = document.querySelectorAll('.robot-fleet-card');
        var panels = document.querySelectorAll('.robot-config-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (card.getAttribute('data-robot-id') === robotId) {
                card.classList.add('robot-fleet-card-active');
            } else {
                card.classList.remove('robot-fleet-card-active');
            }
        }
        for (var index = 0; index < panels.length; index += 1) {
            var panel = panels[index];
            var isActive = panel.getAttribute('data-robot-id') === robotId;
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
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (!programSelect || !memorySelect) {
            return;
        }
        var programOption = programSelect.options[programSelect.selectedIndex];
        var memoryOption = memorySelect.options[memorySelect.selectedIndex];
        var programSize = parseInt(programOption.getAttribute('data-compiled-size') || '0', 10);
        var memorySize = parseInt(memoryOption.getAttribute('data-memory-capacity') || '0', 10);
        if (memorySize <= 0) {
            memorySize = 1;
        }
        var percent = Math.min(100, Math.max(0, (programSize / memorySize) * 100));
        var valueElement = panel.querySelector('.robot-progress-value');
        var barElement = panel.querySelector('.robot-progress-bar');
        if (valueElement) {
            valueElement.textContent = programSize + '/' + memorySize;
        }
        var progressElement = panel.querySelector('.robot-progress');
        if (progressElement) {
            progressElement.classList.toggle('robot-progress-over', programSize > memorySize);
        }
        if (barElement) {
            barElement.style.width = percent.toFixed(1) + '%';
        }
    }

    function robotApplyBlockReason(panel) {
        var nameInput = panel.querySelector('input[name^="robotName"]');
        if (nameInput) {
            var robotName = nameInput.value.trim();
            if (!robotName || robotName.length > 15 || !/^[A-Za-z0-9_]+$/.test(robotName)) {
                return 'Invalid robot name.';
            }
        }
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (programSelect && memorySelect) {
            var selectedProgram = programSelect.options[programSelect.selectedIndex];
            var selectedMemory = memorySelect.options[memorySelect.selectedIndex];
            var programSize = parseInt(selectedProgram.getAttribute('data-compiled-size') || '0', 10);
            var memorySize = parseInt(selectedMemory.getAttribute('data-memory-capacity') || '0', 10);
            if (memorySize > 0 && programSize > memorySize) {
                return 'Not enough memory available.';
            }
        }
        return null;
    }

    function updateRobotProgramHint(panel) {
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var hint = panel.querySelector('.robot-program-hint');
        if (!programSelect || !hint) {
            return;
        }
        var programOption = programSelect.options[programSelect.selectedIndex];
        var hasError = programOption.getAttribute('data-has-compile-error') === '1';
        hint.hidden = !hasError;
        if (hasError) {
            var link = hint.querySelector('a');
            if (link) {
                link.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programOption.value);
            }
        }
    }

    function updateRobotApplyState(panel) {
        if (!panel) {
            return;
        }
        var reason = robotApplyBlockReason(panel);
        var applyButton = panel.querySelector('.robot-btn-primary');
        var hint = panel.querySelector('.robot-action-hint');
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
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        var nameInput = panel.querySelector('input[name^="robotName"]');
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

    function robotUrlId() {
        var search = window.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === 'robotId' && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
    }

    var preferredRobotId = robotUrlId();
    if (preferredRobotId && document.querySelector('.robot-config-panel[data-robot-id="' + preferredRobotId + '"]')) {
        selectRobot(preferredRobotId, false);
    } else {
        var firstCard = document.querySelector('.robot-fleet-card');
        if (firstCard) {
            selectRobot(firstCard.getAttribute('data-robot-id'), false);
        }
    }

    var fleetCards = document.querySelectorAll('.robot-fleet-card');
    for (var fleetIndex = 0; fleetIndex < fleetCards.length; fleetIndex += 1) {
        fleetCards[fleetIndex].addEventListener('click', function(event) {
            var robotId = event.currentTarget.getAttribute('data-robot-id');
            var activePanel = document.querySelector('.robot-config-panel-active');
            if (activePanel
                && activePanel.getAttribute('data-robot-id') !== robotId
                && isPanelDirty(activePanel)) {
                var nameInput = activePanel.querySelector('input[name^="robotName"]');
                var robotName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this robot';
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

    var resetButtons = document.querySelectorAll('.robot-reset-btn');
    for (var resetIndex = 0; resetIndex < resetButtons.length; resetIndex += 1) {
        resetButtons[resetIndex].addEventListener('click', function(event) {
            var panel = event.target.closest('.robot-config-panel');
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
        var panels = document.querySelectorAll('.robot-config-panel');
        for (var unloadIndex = 0; unloadIndex < panels.length; unloadIndex += 1) {
            if (isPanelDirty(panels[unloadIndex])) {
                event.preventDefault();
                event.returnValue = '';
                return;
            }
        }
    });

    var panels = document.querySelectorAll('.robot-config-panel');
    for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
        attachRobotPreviewListeners(panels[panelIndex]);
    }

    function confirmRobotApply(event) {
        var panel = null;
        if (event.submitter) {
            panel = event.submitter.closest('.robot-config-panel');
        }
        if (!panel) {
            panel = document.querySelector('.robot-config-panel-active');
        }
        if (!panel) {
            return;
        }
        var applyButton = panel.querySelector('.robot-btn-primary');
        if (applyButton && applyButton.disabled) {
            event.preventDefault();
            return;
        }
        var nameInput = panel.querySelector('input[name^="robotName"]');
        var robotName = nameInput ? nameInput.value.trim() : 'this robot';
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
            var robotId = panel.getAttribute('data-robot-id');
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

    var robotForm = document.getElementById('robotForm');
    if (robotForm) {
        robotForm.addEventListener('submit', confirmRobotApply);
    }
})();
</script>"#;
