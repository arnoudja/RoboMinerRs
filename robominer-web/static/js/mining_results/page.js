(function() {
    var DEFAULT_INITIAL_VISIBLE = 5;
    var DEFAULT_LOAD_MORE_STEP = 5;
    var visibleRunCount = DEFAULT_INITIAL_VISIBLE;

    function collectMiningResultsQueryParams() {
        var params = {};
        var robotFilter = document.getElementById('miningResultsRobotFilter');
        var areaFilter = document.getElementById('miningResultsAreaFilter');
        var sortFilter = document.getElementById('miningResultsSortFilter');
        var activePanel = document.querySelector('.mining-results-detail-panel-active:not(.mining-results-filter-hidden)');
        if (robotFilter && robotFilter.value) {
            params.robotId = robotFilter.value;
        }
        if (areaFilter && areaFilter.value) {
            params.area = areaFilter.value;
        }
        if (sortFilter && sortFilter.value && sortFilter.value !== 'newest') {
            params.sort = sortFilter.value;
        }
        if (activePanel) {
            params.runId = activePanel.getAttribute('data-run-id');
        }
        return params;
    }

    function syncMiningResultsUrl() {
        window.RoboMinerUrlQuery.sync('miningResults', collectMiningResultsQueryParams());
    }

    function runCardsContainer() {
        return document.querySelector('.mining-results-run-cards');
    }

    function initialVisibleRuns() {
        var container = runCardsContainer();
        if (!container) {
            return DEFAULT_INITIAL_VISIBLE;
        }
        var value = Number(container.getAttribute('data-initial-visible'));
        return value > 0 ? value : DEFAULT_INITIAL_VISIBLE;
    }

    function loadMoreStep() {
        var container = runCardsContainer();
        if (!container) {
            return DEFAULT_LOAD_MORE_STEP;
        }
        var value = Number(container.getAttribute('data-load-more-step'));
        return value > 0 ? value : DEFAULT_LOAD_MORE_STEP;
    }

    function selectMiningResultRun(runId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        var cards = document.querySelectorAll('.mining-results-run-card');
        var panels = document.querySelectorAll('.mining-results-detail-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            var isActive = card.getAttribute('data-run-id') === String(runId)
                && !card.classList.contains('mining-results-filter-hidden')
                && !card.classList.contains('mining-results-run-card-collapsed');
            card.classList.toggle('mining-results-run-card-active', isActive);
        }
        for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            var panel = panels[panelIndex];
            var panelIsActive = panel.getAttribute('data-run-id') === String(runId)
                && !panel.classList.contains('mining-results-filter-hidden');
            panel.classList.toggle('mining-results-detail-panel-active', panelIsActive);
            panel.hidden = !panelIsActive;
        }
        if (updateUrl) {
            syncMiningResultsUrl();
            syncReplayReturnLinks();
        }
    }

    function compareMiningResultElements(left, right, sortBy) {
        if (sortBy === 'reward') {
            return Number(right.getAttribute('data-sort-reward')) - Number(left.getAttribute('data-sort-reward'));
        }
        if (sortBy === 'score') {
            return Number(right.getAttribute('data-sort-score')) - Number(left.getAttribute('data-sort-score'));
        }
        return Number(right.getAttribute('data-sort-end')) - Number(left.getAttribute('data-sort-end'));
    }

    function applyMiningResultsSort() {
        var sortFilter = document.getElementById('miningResultsSortFilter');
        var sortBy = sortFilter ? sortFilter.value : 'newest';
        var container = runCardsContainer();
        if (container) {
            var cards = Array.prototype.slice.call(container.querySelectorAll('.mining-results-run-card'));
            cards.sort(function(left, right) {
                return compareMiningResultElements(left, right, sortBy);
            });
            for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
                container.appendChild(cards[cardIndex]);
            }
        }
        var panelContainer = document.querySelector('.mining-results-detail-panels');
        if (panelContainer) {
            var panels = Array.prototype.slice.call(panelContainer.querySelectorAll('.mining-results-detail-panel'));
            panels.sort(function(left, right) {
                return compareMiningResultElements(left, right, sortBy);
            });
            for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
                panelContainer.appendChild(panels[panelIndex]);
            }
        }
    }

    function syncReplayReturnLinks() {
        var query = window.RoboMinerUrlQuery.buildQueryString(collectMiningResultsQueryParams());
        var links = document.querySelectorAll('.mining-results-replay-link-primary[data-rally-result-id]');
        for (var linkIndex = 0; linkIndex < links.length; linkIndex += 1) {
            var link = links[linkIndex];
            var rallyId = link.getAttribute('data-rally-result-id');
            var href = 'miningResults?rallyResultId=' + encodeURIComponent(rallyId);
            if (query) {
                href += '&returnTo=' + encodeURIComponent(query);
            }
            link.setAttribute('href', href);
        }
    }

    function matchesMiningResultsFilter(element, robotId, areaName) {
        if (robotId && element.getAttribute('data-robot-id') !== robotId) {
            return false;
        }
        if (areaName && element.getAttribute('data-area-name') !== areaName) {
            return false;
        }
        return true;
    }

    function matchingRunCards(robotId, areaName) {
        var cards = document.querySelectorAll('.mining-results-run-card');
        var matching = [];
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (matchesMiningResultsFilter(card, robotId, areaName)) {
                matching.push(card);
            }
        }
        return matching;
    }

    function applyVisibleRunLimit(preferredRunId) {
        var robotFilter = document.getElementById('miningResultsRobotFilter');
        var areaFilter = document.getElementById('miningResultsAreaFilter');
        var robotId = robotFilter ? robotFilter.value : '';
        var areaName = areaFilter ? areaFilter.value : '';
        var matching = matchingRunCards(robotId, areaName);
        if (preferredRunId) {
            for (var matchIndex = 0; matchIndex < matching.length; matchIndex += 1) {
                if (matching[matchIndex].getAttribute('data-run-id') === String(preferredRunId)) {
                    visibleRunCount = Math.max(visibleRunCount, matchIndex + 1);
                    break;
                }
            }
        }
        for (var cardIndex = 0; cardIndex < matching.length; cardIndex += 1) {
            matching[cardIndex].classList.toggle(
                'mining-results-run-card-collapsed',
                cardIndex >= visibleRunCount
            );
        }
        var loadMoreWrap = document.getElementById('miningResultsLoadMoreWrap');
        if (loadMoreWrap) {
            loadMoreWrap.hidden = matching.length <= visibleRunCount;
        }
    }

    function applyMiningResultsFilters(preferredRunId) {
        var robotFilter = document.getElementById('miningResultsRobotFilter');
        var areaFilter = document.getElementById('miningResultsAreaFilter');
        if (!robotFilter || !areaFilter) {
            return;
        }
        var robotId = robotFilter.value;
        var areaName = areaFilter.value;
        var cards = document.querySelectorAll('.mining-results-run-card');
        var panels = document.querySelectorAll('.mining-results-detail-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (matchesMiningResultsFilter(card, robotId, areaName)) {
                card.classList.remove('mining-results-filter-hidden');
            } else {
                card.classList.remove('mining-results-run-card-active');
                card.classList.add('mining-results-filter-hidden');
            }
        }
        for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            var panel = panels[panelIndex];
            if (matchesMiningResultsFilter(panel, robotId, areaName)) {
                panel.classList.remove('mining-results-filter-hidden');
            } else {
                panel.classList.remove('mining-results-detail-panel-active');
                panel.classList.add('mining-results-filter-hidden');
                panel.hidden = true;
            }
        }
        applyVisibleRunLimit(preferredRunId);
        var firstVisibleRunId = null;
        var activeRunId = null;
        for (var visibleIndex = 0; visibleIndex < cards.length; visibleIndex += 1) {
            var visibleCard = cards[visibleIndex];
            if (visibleCard.classList.contains('mining-results-filter-hidden')
                || visibleCard.classList.contains('mining-results-run-card-collapsed')) {
                continue;
            }
            if (!firstVisibleRunId) {
                firstVisibleRunId = visibleCard.getAttribute('data-run-id');
            }
            if (visibleCard.classList.contains('mining-results-run-card-active')) {
                activeRunId = visibleCard.getAttribute('data-run-id');
            }
        }
        var empty = document.getElementById('miningResultsFilterEmpty');
        if (empty) {
            empty.hidden = firstVisibleRunId !== null;
        }
        var nextRunId = null;
        if (preferredRunId && document.querySelector('.mining-results-run-card[data-run-id="' + preferredRunId + '"]:not(.mining-results-filter-hidden):not(.mining-results-run-card-collapsed)')) {
            nextRunId = preferredRunId;
        } else if (activeRunId && document.querySelector('.mining-results-run-card[data-run-id="' + activeRunId + '"]:not(.mining-results-filter-hidden):not(.mining-results-run-card-collapsed)')) {
            nextRunId = activeRunId;
        } else {
            nextRunId = firstVisibleRunId;
        }
        if (nextRunId) {
            selectMiningResultRun(nextRunId, false);
        }
        syncMiningResultsUrl();
        syncReplayReturnLinks();
    }

    var robotFilter = document.getElementById('miningResultsRobotFilter');
    var areaFilter = document.getElementById('miningResultsAreaFilter');
    var sortFilter = document.getElementById('miningResultsSortFilter');
    var loadMoreButton = document.getElementById('miningResultsLoadMore');
    visibleRunCount = initialVisibleRuns();
    if (robotFilter) {
        var preferredRobotId = window.RoboMinerUrlQuery.getParam('robotId');
        if (preferredRobotId) {
            for (var robotIndex = 0; robotIndex < robotFilter.options.length; robotIndex += 1) {
                if (robotFilter.options[robotIndex].value === preferredRobotId) {
                    robotFilter.value = preferredRobotId;
                    break;
                }
            }
        }
    }
    if (areaFilter) {
        var preferredArea = window.RoboMinerUrlQuery.getParam('area');
        if (preferredArea) {
            for (var areaIndex = 0; areaIndex < areaFilter.options.length; areaIndex += 1) {
                if (areaFilter.options[areaIndex].value === preferredArea) {
                    areaFilter.value = preferredArea;
                    break;
                }
            }
        }
    }
    if (sortFilter) {
        var preferredSort = window.RoboMinerUrlQuery.getParam('sort');
        if (preferredSort) {
            for (var sortIndex = 0; sortIndex < sortFilter.options.length; sortIndex += 1) {
                if (sortFilter.options[sortIndex].value === preferredSort) {
                    sortFilter.value = preferredSort;
                    break;
                }
            }
        }
    }
    applyMiningResultsSort();
    applyMiningResultsFilters(window.RoboMinerUrlQuery.getParam('runId'));

    if (robotFilter) {
        robotFilter.addEventListener('change', function() {
            visibleRunCount = initialVisibleRuns();
            applyMiningResultsFilters();
        });
    }
    if (areaFilter) {
        areaFilter.addEventListener('change', function() {
            visibleRunCount = initialVisibleRuns();
            applyMiningResultsFilters();
        });
    }
    if (sortFilter) {
        sortFilter.addEventListener('change', function() {
            applyMiningResultsSort();
            applyMiningResultsFilters();
        });
    }
    if (loadMoreButton) {
        loadMoreButton.addEventListener('click', function() {
            visibleRunCount += loadMoreStep();
            applyVisibleRunLimit();
        });
    }

    var runCards = document.querySelectorAll('.mining-results-run-card');
    for (var runIndex = 0; runIndex < runCards.length; runIndex += 1) {
        runCards[runIndex].addEventListener('click', function(event) {
            selectMiningResultRun(event.currentTarget.getAttribute('data-run-id'));
        });
    }
})();
