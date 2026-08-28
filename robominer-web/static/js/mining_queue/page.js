(function() {
    var FRAGMENT_PARAM = 'queue';
    var REFRESH_DEBOUNCE_MS = 300;
    var CLAIM_REFRESH_BACKOFF_MS = [1000, 2000, 4000, 8000];
    var CREDIT_FEEDBACK_MS = 5000;

    var pageRoot = document.querySelector('.mining-queue-page');
    var STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-area-storage-key') || 'robominer.miningQueue.areaSelections'
        : 'robominer.miningQueue.areaSelections';

    var ctx = {
        pageRoot: pageRoot,
        STORAGE_KEY: STORAGE_KEY,
        FRAGMENT_PARAM: FRAGMENT_PARAM,
        REFRESH_DEBOUNCE_MS: REFRESH_DEBOUNCE_MS,
        CLAIM_REFRESH_BACKOFF_MS: CLAIM_REFRESH_BACKOFF_MS,
        CREDIT_FEEDBACK_MS: CREDIT_FEEDBACK_MS,
        timerIntervals: [],
        resizeObserver: null,
        refreshDebounceTimer: null,
        refreshInFlight: false,
        refreshPending: false,
        claimRefreshAttempt: 0,
        claimRefreshTimer: null,
        claimBaselineSignature: null,
        claimBaselineAmounts: null,
        creditFeedbackTimer: null,
        inspectorSelect: document.getElementById('infoMiningAreaId'),
        updateClearButtonLabel: null,
        init: null,
        refreshQueue: null,
        collectQueueQueryParams: collectQueueQueryParams,
        buildFragmentUrl: buildFragmentUrl,
        writeStoredAreaSelections: writeStoredAreaSelections,
        restoreAreaSelectionsFromStorage: restoreAreaSelectionsFromStorage,
    };

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

    function buildFragmentUrl(extraParams) {
        var params = collectQueueQueryParams();
        params.fragment = FRAGMENT_PARAM;
        if (extraParams) {
            Object.keys(extraParams).forEach(function(name) {
                params[name] = extraParams[name];
            });
        }
        var query = window.RoboMinerUrlQuery.buildQueryString(params);
        return query ? 'miningQueue?' + query : 'miningQueue?fragment=' + FRAGMENT_PARAM;
    }

    function clearTimers() {
        for (var index = 0; index < ctx.timerIntervals.length; index += 1) {
            window.clearInterval(ctx.timerIntervals[index]);
        }
        ctx.timerIntervals = [];
    }

    function disconnectObserver() {
        if (ctx.resizeObserver) {
            ctx.resizeObserver.disconnect();
            ctx.resizeObserver = null;
        }
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
        if (ctx.inspectorSelect && applyStoredAreaSelection(ctx.inspectorSelect, stored.infoMiningAreaId)) {
            changed = true;
        }
        var robotAreaSelects = document.querySelectorAll('select[name^="miningArea"]');
        for (var restoreIndex = 0; restoreIndex < robotAreaSelects.length; restoreIndex += 1) {
            var robotSelect = robotAreaSelects[restoreIndex];
            if (applyStoredAreaSelection(robotSelect, stored[robotSelect.name])) {
                view.updateRobotEnqueueState(robotSelect);
                changed = true;
            }
        }
        if (changed && ctx.inspectorSelect && ctx.inspectorSelect.value) {
            view.syncInspectorArea(ctx.inspectorSelect.value);
        } else if (changed) {
            writeStoredAreaSelections();
        }
    }

    function init(options) {
        options = options || {};
        clearTimers();
        disconnectObserver();
        view.initView(options);
    }

    ctx.init = init;

    var view = window.RoboMinerMiningQueueInstall.view(ctx);
    var claimPoll = window.RoboMinerMiningQueueInstall.claimPoll(ctx, view);
    var actions = window.RoboMinerMiningQueueInstall.actions(ctx, view);
    ctx.refreshQueue = claimPoll.refreshQueue;

    document.addEventListener('change', function(event) {
        var checkbox = event.target.closest('.mining-queue-item-check');
        if (checkbox) {
            var checkboxForm = checkbox.closest('.mining-queue-card');
            if (checkboxForm) {
                ctx.updateClearButtonLabel(checkboxForm);
            }
            return;
        }

        var robotSelect = event.target.closest('.mining-queue-card select[name^="miningArea"]');
        if (!robotSelect) {
            return;
        }
        var areaId = robotSelect.value;
        view.updateRobotEnqueueState(robotSelect);
        if (ctx.inspectorSelect && areaId) {
            ctx.inspectorSelect.value = areaId;
            view.syncInspectorArea(areaId);
        } else {
            writeStoredAreaSelections();
        }
    });

    document.addEventListener('click', function(event) {
        var removeButton = event.target.closest('.mining-queue-remove-btn');
        if (removeButton) {
            event.preventDefault();
            actions.removeQueuedRun(removeButton);
            return;
        }
        var clearButton = event.target.closest('.mining-queue-clear-btn');
        if (clearButton) {
            event.preventDefault();
            actions.clearQueuedRuns(clearButton);
        }
    });

    document.addEventListener('submit', function(event) {
        var form = event.target.closest('.mining-queue-card');
        if (!form || form.tagName !== 'FORM') {
            return;
        }
        event.preventDefault();
        var formData = new FormData(form);
        if (event.submitter && event.submitter.name) {
            formData.set(event.submitter.name, event.submitter.value);
        }
        view.fetchFragment('POST', buildFragmentUrl(), formData).catch(function() {
            form.submit();
        });
    });

    if (ctx.inspectorSelect) {
        ctx.inspectorSelect.addEventListener('change', function() {
            view.syncInspectorArea(ctx.inspectorSelect.value);
        });
    }

    init();

    window.RoboMinerMiningQueuePage = {
        FRAGMENT_PARAM: FRAGMENT_PARAM,
        REFRESH_DEBOUNCE_MS: REFRESH_DEBOUNCE_MS,
        CLAIM_REFRESH_BACKOFF_MS: CLAIM_REFRESH_BACKOFF_MS,
        CREDIT_FEEDBACK_MS: CREDIT_FEEDBACK_MS,
        applyFragment: view.applyFragment,
        applyHudFragment: view.applyHudFragment,
        hasFinishingRuns: claimPoll.hasFinishingRuns,
        walletSignature: claimPoll.walletSignature,
        parseWalletAmounts: claimPoll.parseWalletAmounts,
        walletCreditDeltas: claimPoll.walletCreditDeltas,
        showWalletCreditFeedback: claimPoll.showWalletCreditFeedback,
        buildFragmentUrl: buildFragmentUrl,
        formDataToUrlEncoded: view.formDataToUrlEncoded,
        init: init,
        refreshQueue: claimPoll.refreshQueue,
        performRefresh: claimPoll.performRefresh,
    };
})();
