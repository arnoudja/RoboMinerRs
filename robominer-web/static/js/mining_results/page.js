(function() {
    const DEFAULT_INITIAL_VISIBLE = 5;
    const DEFAULT_LOAD_MORE_STEP = 5;
    let visibleRunCount = DEFAULT_INITIAL_VISIBLE;

    function collectMiningResultsQueryParams() {
        const params = {};
        const robotFilter = document.getElementById('miningResultsRobotFilter');
        const areaFilter = document.getElementById('miningResultsAreaFilter');
        const sortFilter = document.getElementById('miningResultsSortFilter');
        const activePanel = document.querySelector('.mining-results-detail-panel-active:not(.mining-results-filter-hidden)');
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
        const container = runCardsContainer();
        if (!container) {
            return DEFAULT_INITIAL_VISIBLE;
        }
        const value = Number(container.getAttribute('data-initial-visible'));
        return value > 0 ? value : DEFAULT_INITIAL_VISIBLE;
    }

    function loadMoreStep() {
        const container = runCardsContainer();
        if (!container) {
            return DEFAULT_LOAD_MORE_STEP;
        }
        const value = Number(container.getAttribute('data-load-more-step'));
        return value > 0 ? value : DEFAULT_LOAD_MORE_STEP;
    }

    function selectMiningResultRun(runId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        const cards = document.querySelectorAll('.mining-results-run-card');
        const panels = document.querySelectorAll('.mining-results-detail-panel');
        for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            const card = cards[cardIndex];
            const isActive = card.getAttribute('data-run-id') === String(runId)
                && !card.classList.contains('mining-results-filter-hidden')
                && !card.classList.contains('mining-results-run-card-collapsed');
            card.classList.toggle('mining-results-run-card-active', isActive);
        }
        for (let panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            const panel = panels[panelIndex];
            const panelIsActive = panel.getAttribute('data-run-id') === String(runId)
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
        const sortFilter = document.getElementById('miningResultsSortFilter');
        const sortBy = sortFilter ? sortFilter.value : 'newest';
        const container = runCardsContainer();
        if (container) {
            const cards = Array.prototype.slice.call(container.querySelectorAll('.mining-results-run-card'));
            cards.sort(function(left, right) {
                return compareMiningResultElements(left, right, sortBy);
            });
            for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
                container.appendChild(cards[cardIndex]);
            }
        }
        const panelContainer = document.querySelector('.mining-results-detail-panels');
        if (panelContainer) {
            const panels = Array.prototype.slice.call(panelContainer.querySelectorAll('.mining-results-detail-panel'));
            panels.sort(function(left, right) {
                return compareMiningResultElements(left, right, sortBy);
            });
            for (let panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
                panelContainer.appendChild(panels[panelIndex]);
            }
        }
    }

    function syncReplayReturnLinks() {
        const query = window.RoboMinerUrlQuery.buildQueryString(collectMiningResultsQueryParams());
        const links = document.querySelectorAll('.mining-results-replay-link-primary[data-rally-result-id]');
        for (let linkIndex = 0; linkIndex < links.length; linkIndex += 1) {
            const link = links[linkIndex];
            const rallyId = link.getAttribute('data-rally-result-id');
            let href = 'miningResults?rallyResultId=' + encodeURIComponent(rallyId);
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
        const cards = document.querySelectorAll('.mining-results-run-card');
        const matching = [];
        for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            const card = cards[cardIndex];
            if (matchesMiningResultsFilter(card, robotId, areaName)) {
                matching.push(card);
            }
        }
        return matching;
    }

    function applyVisibleRunLimit(preferredRunId) {
        const robotFilter = document.getElementById('miningResultsRobotFilter');
        const areaFilter = document.getElementById('miningResultsAreaFilter');
        const robotId = robotFilter ? robotFilter.value : '';
        const areaName = areaFilter ? areaFilter.value : '';
        const matching = matchingRunCards(robotId, areaName);
        if (preferredRunId) {
            for (let matchIndex = 0; matchIndex < matching.length; matchIndex += 1) {
                if (matching[matchIndex].getAttribute('data-run-id') === String(preferredRunId)) {
                    visibleRunCount = Math.max(visibleRunCount, matchIndex + 1);
                    break;
                }
            }
        }
        for (let cardIndex = 0; cardIndex < matching.length; cardIndex += 1) {
            matching[cardIndex].classList.toggle(
                'mining-results-run-card-collapsed',
                cardIndex >= visibleRunCount
            );
        }
        const loadMoreWrap = document.getElementById('miningResultsLoadMoreWrap');
        if (loadMoreWrap) {
            loadMoreWrap.hidden = matching.length <= visibleRunCount;
        }
    }

    function applyMiningResultsFilters(preferredRunId) {
        const robotFilter = document.getElementById('miningResultsRobotFilter');
        const areaFilter = document.getElementById('miningResultsAreaFilter');
        if (!robotFilter || !areaFilter) {
            return;
        }
        const robotId = robotFilter.value;
        const areaName = areaFilter.value;
        const cards = document.querySelectorAll('.mining-results-run-card');
        const panels = document.querySelectorAll('.mining-results-detail-panel');
        for (let cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            const card = cards[cardIndex];
            if (matchesMiningResultsFilter(card, robotId, areaName)) {
                card.classList.remove('mining-results-filter-hidden');
            } else {
                card.classList.remove('mining-results-run-card-active');
                card.classList.add('mining-results-filter-hidden');
            }
        }
        for (let panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            const panel = panels[panelIndex];
            if (matchesMiningResultsFilter(panel, robotId, areaName)) {
                panel.classList.remove('mining-results-filter-hidden');
            } else {
                panel.classList.remove('mining-results-detail-panel-active');
                panel.classList.add('mining-results-filter-hidden');
                panel.hidden = true;
            }
        }
        applyVisibleRunLimit(preferredRunId);
        let firstVisibleRunId = null;
        let activeRunId = null;
        for (let visibleIndex = 0; visibleIndex < cards.length; visibleIndex += 1) {
            const visibleCard = cards[visibleIndex];
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
        const empty = document.getElementById('miningResultsFilterEmpty');
        if (empty) {
            empty.hidden = firstVisibleRunId !== null;
        }
        let nextRunId = null;
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

    const robotFilter = document.getElementById('miningResultsRobotFilter');
    const areaFilter = document.getElementById('miningResultsAreaFilter');
    const sortFilter = document.getElementById('miningResultsSortFilter');
    const loadMoreButton = document.getElementById('miningResultsLoadMore');
    visibleRunCount = initialVisibleRuns();
    if (robotFilter) {
        const preferredRobotId = window.RoboMinerUrlQuery.getParam('robotId');
        if (preferredRobotId) {
            for (let robotIndex = 0; robotIndex < robotFilter.options.length; robotIndex += 1) {
                if (robotFilter.options[robotIndex].value === preferredRobotId) {
                    robotFilter.value = preferredRobotId;
                    break;
                }
            }
        }
    }
    if (areaFilter) {
        const preferredArea = window.RoboMinerUrlQuery.getParam('area');
        if (preferredArea) {
            for (let areaIndex = 0; areaIndex < areaFilter.options.length; areaIndex += 1) {
                if (areaFilter.options[areaIndex].value === preferredArea) {
                    areaFilter.value = preferredArea;
                    break;
                }
            }
        }
    }
    if (sortFilter) {
        const preferredSort = window.RoboMinerUrlQuery.getParam('sort');
        if (preferredSort) {
            for (let sortIndex = 0; sortIndex < sortFilter.options.length; sortIndex += 1) {
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

    const runCards = document.querySelectorAll('.mining-results-run-card');
    for (let runIndex = 0; runIndex < runCards.length; runIndex += 1) {
        runCards[runIndex].addEventListener('click', function(event) {
            selectMiningResultRun(event.currentTarget.getAttribute('data-run-id'));
        });
    }
})();
