pub(super) const MINING_QUEUE_PAGE_SCRIPT: &str = r#"<script>
(function() {
    var pageRoot = document.querySelector('.mining-queue-page');
    var STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-area-storage-key') || 'robominer.miningQueue.areaSelections'
        : 'robominer.miningQueue.areaSelections';

    function readStoredAreaSelections() {
        try {
            var raw = window.sessionStorage.getItem(STORAGE_KEY);
            if (!raw) {
                return null;
            }
            return JSON.parse(raw);
        } catch (error) {
            return null;
        }
    }

    function writeStoredAreaSelections() {
        try {
            var stored = {};
            var selects = document.querySelectorAll('select[name="infoMiningAreaId"], select[name^="miningArea"]');
            for (var index = 0; index < selects.length; index += 1) {
                var select = selects[index];
                if (select.name && select.value) {
                    stored[select.name] = select.value;
                }
            }
            window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(stored));
        } catch (error) {
        }
    }

    function urlHasAreaSelectionParams() {
        var search = window.location.search;
        if (!search) {
            return false;
        }
        var params = search.substring(1).split('&');
        for (var paramIndex = 0; paramIndex < params.length; paramIndex += 1) {
            var name = decodeURIComponent(params[paramIndex].split('=')[0]);
            if (name === 'infoMiningAreaId' || name.indexOf('miningArea') === 0) {
                return true;
            }
        }
        return false;
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
        var params = [];
        var selects = document.querySelectorAll('select[name="infoMiningAreaId"], select[name^="miningArea"]');
        for (var index = 0; index < selects.length; index += 1) {
            var select = selects[index];
            if (select.value) {
                params.push(encodeURIComponent(select.name) + '=' + encodeURIComponent(select.value));
            }
        }
        return params.join('&');
    }

    function refreshQueue() {
        var query = collectQueueQueryParams();
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
        var query = collectQueueQueryParams();
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', query ? 'miningQueue?' + query : 'miningQueue');
        }
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

    function submitQueuedRunRemoval(form, queueItemId) {
        var staleInputs = form.querySelectorAll('input[data-mining-queue-remove="true"]');
        for (var staleIndex = 0; staleIndex < staleInputs.length; staleIndex += 1) {
            staleInputs[staleIndex].remove();
        }
        function addHidden(name, value) {
            var input = document.createElement('input');
            input.type = 'hidden';
            input.name = name;
            input.value = value;
            input.setAttribute('data-mining-queue-remove', 'true');
            form.appendChild(input);
        }
        addHidden('selectedQueueItemId', queueItemId);
        addHidden('submitType', 'remove');
        form.submit();
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

    window.miningQueueRemoveRun = removeQueuedRun;

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
</script>"#;
