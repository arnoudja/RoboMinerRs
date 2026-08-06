(function() {
    var pageRoot = document.querySelector('.mining-queue-page');
    var STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-area-storage-key') || 'robominer.miningQueue.areaSelections'
        : 'robominer.miningQueue.areaSelections';

    function readStoredAreaSelections() {
        return window.RoboMinerSessionStore.readJson(STORAGE_KEY);
    }

    function writeStoredAreaSelections() {
        var stored = {};
        var selects = document.querySelectorAll('select[name="infoMiningAreaId"], select[name^="miningArea"]');
        for (var index = 0; index < selects.length; index += 1) {
            var select = selects[index];
            if (select.name && select.value) {
                stored[select.name] = select.value;
            }
        }
        window.RoboMinerSessionStore.writeJson(STORAGE_KEY, stored);
    }

    function areaSelectionParamNames() {
        var names = ['infoMiningAreaId'];
        var selects = document.querySelectorAll('select[name^="miningArea"]');
        for (var index = 0; index < selects.length; index += 1) {
            if (selects[index].name) {
                names.push(selects[index].name);
            }
        }
        return names;
    }

    function urlHasAreaSelectionParams() {
        return window.RoboMinerUrlQuery.hasAnyParam(areaSelectionParamNames());
    }

    function selectHasOption(select, areaId) {
        var value = String(areaId);
        for (var optionIndex = 0; optionIndex < select.options.length; optionIndex += 1) {
            if (select.options[optionIndex].value === value) {
                return true;
            }
        }
        return false;
    }

    function applyStoredAreaSelection(select, areaId) {
        if (!select || !areaId || !selectHasOption(select, areaId)) {
            return false;
        }
        if (select.value === String(areaId)) {
            return false;
        }
        select.value = String(areaId);
        return true;
    }

    function formatTimeLeft(seconds) {
        var secondsLeft = Math.max(0, Math.floor(seconds));
        var displaySeconds = secondsLeft % 60;
        var displayMinutes = Math.floor(secondsLeft / 60) % 60;
        var displayHours = Math.floor(secondsLeft / 3600);
        var result = displayHours > 0 ? displayHours + ':' : '';
        if (displayMinutes < 10 && displayHours > 0) {
            result += '0';
        }
        result += displayMinutes + ':';
        if (displaySeconds < 10) {
            result += '0';
        }
        return result + displaySeconds;
    }

    function collectQueueQueryParams() {
        var params = {};
        var selects = document.querySelectorAll('select[name="infoMiningAreaId"], select[name^="miningArea"]');
        for (var index = 0; index < selects.length; index += 1) {
            var select = selects[index];
            if (select.name && select.value) {
                params[select.name] = select.value;
            }
        }
        return params;
    }

    function refreshQueue() {
        var query = window.RoboMinerUrlQuery.buildQueryString(collectQueueQueryParams());
        window.location.replace(query ? 'miningQueue?' + query : 'miningQueue');
    }

    function showMiningAreaDetails(areaId) {
        var panels = document.querySelectorAll('tbody.mining-queue-area-panel');
        for (var index = 0; index < panels.length; index += 1) {
            var panel = panels[index];
            if (panel.id === 'miningAreaDetails' + areaId) {
                panel.classList.add('mining-queue-area-panel-active');
            } else {
                panel.classList.remove('mining-queue-area-panel-active');
            }
        }
    }

    function syncInspectorArea(areaId) {
        showMiningAreaDetails(areaId);
        window.RoboMinerUrlQuery.sync('miningQueue', collectQueueQueryParams());
        writeStoredAreaSelections();
    }

    function restoreAreaSelectionsFromStorage() {
        if (urlHasAreaSelectionParams()) {
            writeStoredAreaSelections();
            return;
        }
        var stored = readStoredAreaSelections();
        if (!stored) {
            return;
        }
        var changed = false;
        if (inspectorSelect && applyStoredAreaSelection(inspectorSelect, stored.infoMiningAreaId)) {
            changed = true;
        }
        for (var restoreIndex = 0; restoreIndex < robotAreaSelects.length; restoreIndex += 1) {
            var robotSelect = robotAreaSelects[restoreIndex];
            if (applyStoredAreaSelection(robotSelect, stored[robotSelect.name])) {
                updateRobotEnqueueState(robotSelect);
                changed = true;
            }
        }
        if (changed && inspectorSelect && inspectorSelect.value) {
            syncInspectorArea(inspectorSelect.value);
        } else if (changed) {
            writeStoredAreaSelections();
        }
    }

    function submitFormWithHiddenFields(form, markerAttr, fields) {
        var staleInputs = form.querySelectorAll('input[' + markerAttr + '="true"]');
        for (var staleIndex = 0; staleIndex < staleInputs.length; staleIndex += 1) {
            staleInputs[staleIndex].remove();
        }
        Object.keys(fields).forEach(function(name) {
            var value = fields[name];
            var values = Array.isArray(value) ? value : [value];
            for (var valueIndex = 0; valueIndex < values.length; valueIndex += 1) {
                var input = document.createElement('input');
                input.type = 'hidden';
                input.name = name;
                input.value = values[valueIndex];
                input.setAttribute(markerAttr, 'true');
                form.appendChild(input);
            }
        });
        form.submit();
    }

    function updateClearButtonLabel(form) {
        var clearButton = form.querySelector('.mining-queue-clear-btn');
        if (!clearButton) {
            return;
        }
        var checked = form.querySelectorAll('.mining-queue-item-check:checked');
        clearButton.textContent = checked.length > 0 ? 'Clear selected' : 'Clear queue';
    }

    function submitQueuedRunRemoval(form, queueItemId) {
        submitFormWithHiddenFields(form, 'data-mining-queue-remove', {
            selectedQueueItemId: queueItemId,
            submitType: 'remove'
        });
    }

    function removeQueuedRun(button) {
        var form = button.closest('.mining-queue-card');
        if (!form) {
            return;
        }
        var queueItemId = button.getAttribute('data-queue-item-id');
        if (!queueItemId) {
            return;
        }
        var row = button.closest('.mining-queue-run-row');
        var area = row ? row.querySelector('.mining-queue-run-area') : null;
        var areaName = area ? area.textContent.trim() : 'queued run';
        var message = 'Remove queued run in ' + areaName + '?';
        if (typeof window.robominerConfirm === 'function') {
            window.robominerConfirm(message, function(confirmed) {
                if (!confirmed) {
                    return;
                }
                submitQueuedRunRemoval(form, queueItemId);
            });
            return;
        }
        if (window.confirm(message)) {
            submitQueuedRunRemoval(form, queueItemId);
        }
    }

    function readClearConfig() {
        var empty = { ores: {}, areaCosts: {}, initialOreWalletMax: 0 };
        var configEl = document.getElementById('mining-queue-clear-config');
        if (!configEl) {
            return empty;
        }
        try {
            var parsed = JSON.parse(configEl.textContent || '{}');
            if (!parsed || typeof parsed !== 'object') {
                return empty;
            }
            return parsed;
        } catch (error) {
            return empty;
        }
    }

    function submitQueueClear(form, clearMode, selectedQueueItemIds) {
        var fields = {
            submitType: 'clear',
            clearMode: clearMode
        };
        if (selectedQueueItemIds && selectedQueueItemIds.length > 0) {
            fields.selectedQueueItemId = selectedQueueItemIds;
        }
        submitFormWithHiddenFields(form, 'data-mining-queue-clear', fields);
    }

    function clearQueuedRuns(button) {
        var form = button.closest('.mining-queue-card');
        if (!form || button.disabled) {
            return;
        }
        var checked = form.querySelectorAll('.mining-queue-item-check:checked');
        var selectedOnly = checked.length > 0;
        var targets = selectedOnly
            ? checked
            : form.querySelectorAll('.mining-queue-remove-btn[data-queue-item-id]');
        if (!targets.length) {
            return;
        }
        var clearHelpers = window.RoboMinerMiningQueueClear;
        if (!clearHelpers) {
            return;
        }
        var config = readClearConfig();
        var areaIds = [];
        var selectedQueueItemIds = [];
        for (var targetIndex = 0; targetIndex < targets.length; targetIndex += 1) {
            areaIds.push(targets[targetIndex].getAttribute('data-mining-area-id'));
            if (selectedOnly) {
                selectedQueueItemIds.push(targets[targetIndex].getAttribute('data-queue-item-id'));
            }
        }
        var wouldLoseOre = clearHelpers.clearingAllWouldLoseOre(config, areaIds);

        function proceed(clearMode) {
            submitQueueClear(form, clearMode, selectedOnly ? selectedQueueItemIds : null);
        }

        if (wouldLoseOre) {
            var lossMessage = selectedOnly
                ? 'Clearing the selected runs would refund ore past your wallet maximum, so some ore would be lost. Clear selected runs anyway, or only clear runs that fit without losing ore?'
                : 'Clearing this queue would refund ore past your wallet maximum, so some ore would be lost. Clear all queued runs anyway, or only clear runs that fit without losing ore?';
            // Three-way choice requires robominerConfirmChoice; do not degrade to
            // window.confirm (that cannot offer the safe-clear path).
            if (typeof window.robominerConfirmChoice === 'function') {
                window.robominerConfirmChoice(
                    lossMessage,
                    {
                        confirmLabel: selectedOnly ? 'Clear selected' : 'Clear all',
                        altLabel: 'Clear without losing ore'
                    },
                    function(result) {
                        if (result === 'confirm') {
                            proceed('all');
                        } else if (result === 'alt') {
                            proceed('safe');
                        }
                    }
                );
            }
            return;
        }

        var message = selectedOnly
            ? 'Clear selected queued runs for this robot?'
            : 'Clear all queued runs for this robot?';
        if (typeof window.robominerConfirm === 'function') {
            window.robominerConfirm(message, function(confirmed) {
                if (!confirmed) {
                    return;
                }
                proceed('all');
            });
            return;
        }
        if (window.confirm(message)) {
            proceed('all');
        }
    }

    document.addEventListener('change', function(event) {
        var checkbox = event.target.closest('.mining-queue-item-check');
        if (!checkbox) {
            return;
        }
        var form = checkbox.closest('.mining-queue-card');
        if (form) {
            updateClearButtonLabel(form);
        }
    });

    document.addEventListener('click', function(event) {
        var removeButton = event.target.closest('.mining-queue-remove-btn');
        if (removeButton) {
            event.preventDefault();
            removeQueuedRun(removeButton);
            return;
        }
        var clearButton = event.target.closest('.mining-queue-clear-btn');
        if (clearButton) {
            event.preventDefault();
            clearQueuedRuns(clearButton);
        }
    });

    function updateRobotEnqueueState(select) {
        var form = select.closest('.mining-queue-card');
        if (!form) {
            return;
        }
        var selectedOption = select.options[select.selectedIndex];
        var blockReason = selectedOption ? selectedOption.getAttribute('data-block-reason') : '';
        if (blockReason === null) {
            blockReason = '';
        }
        var disabled = blockReason.length > 0;
        var buttons = form.querySelectorAll('button[name="submitType"][value="add"], button[name="submitType"][value="fill"]');
        for (var buttonIndex = 0; buttonIndex < buttons.length; buttonIndex += 1) {
            var button = buttons[buttonIndex];
            button.disabled = disabled;
            if (disabled) {
                button.setAttribute('title', blockReason);
            } else {
                button.removeAttribute('title');
            }
        }
        var hint = form.querySelector('.mining-queue-action-hint');
        if (hint) {
            hint.textContent = blockReason;
            hint.hidden = !disabled;
        }
    }

    var inspectorSelect = document.getElementById('infoMiningAreaId');
    if (inspectorSelect) {
        inspectorSelect.addEventListener('change', function() {
            syncInspectorArea(inspectorSelect.value);
        });
    }

    var robotAreaSelects = document.querySelectorAll('select[name^="miningArea"]');
    for (var selectIndex = 0; selectIndex < robotAreaSelects.length; selectIndex += 1) {
        updateRobotEnqueueState(robotAreaSelects[selectIndex]);
        robotAreaSelects[selectIndex].addEventListener('change', function(event) {
            var areaId = event.target.value;
            updateRobotEnqueueState(event.target);
            if (inspectorSelect && areaId) {
                inspectorSelect.value = areaId;
                syncInspectorArea(areaId);
            } else {
                writeStoredAreaSelections();
            }
        });
    }

    try {
        restoreAreaSelectionsFromStorage();
    } catch (error) {
    }

    function startTimer(cell) {
        var seconds = Number(cell.getAttribute('data-seconds-left'));
        if (!isFinite(seconds)) {
            return;
        }
        var refreshOnComplete = cell.getAttribute('data-refresh-on-complete') === 'true';
        var progressTotal = Number(cell.getAttribute('data-progress-total'));
        function updateProgress(secondsLeft) {
            if (!isFinite(progressTotal) || progressTotal <= 0) {
                return;
            }
            var run = cell.closest('.mining-queue-run-active');
            if (!run) {
                return;
            }
            var progressBar = run.querySelector('.mining-queue-progress-bar');
            if (!progressBar) {
                return;
            }
            var elapsed = progressTotal - Math.max(0, secondsLeft);
            var percent = Math.min(100, Math.max(0, (elapsed / progressTotal) * 100));
            progressBar.style.width = percent + '%';
        }
        if (seconds <= 0) {
            updateProgress(0);
            if (refreshOnComplete) {
                refreshQueue();
            }
            return;
        }
        var startTime = Date.now();
        updateProgress(seconds);
        var interval = window.setInterval(function() {
            var secondsLeft = seconds - ((Date.now() - startTime) / 1000);
            if (secondsLeft > 0) {
                cell.textContent = formatTimeLeft(secondsLeft);
                updateProgress(secondsLeft);
                return;
            }
            window.clearInterval(interval);
            cell.textContent = formatTimeLeft(0);
            updateProgress(0);
            if (refreshOnComplete) {
                refreshQueue();
            }
        }, 200);
        cell.textContent = formatTimeLeft(seconds);
    }

    function areaNameOverflows(area) {
        var target = area.querySelector('a') || area;
        return target.scrollWidth > target.clientWidth + 1;
    }

    function syncQueuedStatusVisibility(row) {
        var area = row.querySelector('.mining-queue-run-area');
        var status = row.querySelector('.mining-queue-status-queued');
        if (!area || !status) {
            return;
        }
        status.classList.remove('mining-queue-status-compact-hidden');
        if (areaNameOverflows(area)) {
            status.classList.add('mining-queue-status-compact-hidden');
        }
    }

    function syncAllQueuedStatusVisibility() {
        var rows = document.querySelectorAll('.mining-queue-run-row');
        for (var rowIndex = 0; rowIndex < rows.length; rowIndex += 1) {
            syncQueuedStatusVisibility(rows[rowIndex]);
        }
    }

    function observeQueuedStatusVisibility() {
        function scheduleSync() {
            window.requestAnimationFrame(function() {
                window.requestAnimationFrame(syncAllQueuedStatusVisibility);
            });
        }
        scheduleSync();
        window.addEventListener('resize', scheduleSync);
        if (typeof ResizeObserver === 'undefined') {
            return;
        }
        var observer = new ResizeObserver(scheduleSync);
        var containers = document.querySelectorAll('.mining-queue-card, .mining-queue-run, .mining-queue-upcoming-list li');
        for (var containerIndex = 0; containerIndex < containers.length; containerIndex += 1) {
            observer.observe(containers[containerIndex]);
        }
    }

    observeQueuedStatusVisibility();

    var cells = document.querySelectorAll('.miningqueuetime[data-seconds-left]');
    for (var cellIndex = 0; cellIndex < cells.length; cellIndex += 1) {
        startTimer(cells[cellIndex]);
    }
})();
