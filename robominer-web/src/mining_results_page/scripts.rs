pub(super) const MINING_RESULTS_PAGE_SCRIPT: &str = r#"<script>
(function() {
    function collectMiningResultsQueryParams() {
        var params = [];
        var robotFilter = document.getElementById('miningResultsRobotFilter');
        var areaFilter = document.getElementById('miningResultsAreaFilter');
        var sortFilter = document.getElementById('miningResultsSortFilter');
        var activePanel = document.querySelector('.mining-results-detail-panel-active:not(.mining-results-filter-hidden)');
        if (robotFilter && robotFilter.value) {
            params.push(encodeURIComponent('robotId') + '=' + encodeURIComponent(robotFilter.value));
        }
        if (areaFilter && areaFilter.value) {
            params.push(encodeURIComponent('area') + '=' + encodeURIComponent(areaFilter.value));
        }
        if (sortFilter && sortFilter.value && sortFilter.value !== 'newest') {
            params.push(encodeURIComponent('sort') + '=' + encodeURIComponent(sortFilter.value));
        }
        if (activePanel) {
            params.push(encodeURIComponent('runId') + '=' + encodeURIComponent(activePanel.getAttribute('data-run-id')));
        }
        return params.join('&');
    }

    function syncMiningResultsUrl() {
        var query = collectMiningResultsQueryParams();
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', query ? 'miningResults?' + query : 'miningResults');
        }
    }

    function miningResultsUrlParam(name) {
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

    function selectMiningResultRun(runId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        var cards = document.querySelectorAll('.mining-results-run-card');
        var panels = document.querySelectorAll('.mining-results-detail-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            var isActive = card.getAttribute('data-run-id') === String(runId)
                && !card.classList.contains('mining-results-filter-hidden');
            card.classList.toggle('mining-results-run-card-active', isActive);
        }
        for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            var panel = panels[panelIndex];
            var isActive = panel.getAttribute('data-run-id') === String(runId)
                && !panel.classList.contains('mining-results-filter-hidden');
            panel.classList.toggle('mining-results-detail-panel-active', isActive);
            panel.hidden = !isActive;
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
        var cardContainers = document.querySelectorAll('.mining-results-run-cards');
        for (var containerIndex = 0; containerIndex < cardContainers.length; containerIndex += 1) {
            var container = cardContainers[containerIndex];
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
        var query = collectMiningResultsQueryParams();
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
        var groups = document.querySelectorAll('.mining-results-robot-group');
        var firstVisibleRunId = null;
        var activeRunId = null;
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (matchesMiningResultsFilter(card, robotId, areaName)) {
                card.classList.remove('mining-results-filter-hidden');
                if (!firstVisibleRunId) {
                    firstVisibleRunId = card.getAttribute('data-run-id');
                }
                if (card.classList.contains('mining-results-run-card-active')) {
                    activeRunId = card.getAttribute('data-run-id');
                }
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
        for (var groupIndex = 0; groupIndex < groups.length; groupIndex += 1) {
            var group = groups[groupIndex];
            var visibleCard = group.querySelector('.mining-results-run-card:not(.mining-results-filter-hidden)');
            group.hidden = !visibleCard;
        }
        var empty = document.getElementById('miningResultsFilterEmpty');
        if (empty) {
            empty.hidden = firstVisibleRunId !== null;
        }
        var nextRunId = null;
        if (preferredRunId && document.querySelector('.mining-results-run-card[data-run-id="' + preferredRunId + '"]:not(.mining-results-filter-hidden)')) {
            nextRunId = preferredRunId;
        } else if (activeRunId && document.querySelector('.mining-results-run-card[data-run-id="' + activeRunId + '"]:not(.mining-results-filter-hidden)')) {
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
    if (robotFilter) {
        var preferredRobotId = miningResultsUrlParam('robotId');
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
        var preferredArea = miningResultsUrlParam('area');
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
        var preferredSort = miningResultsUrlParam('sort');
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
    applyMiningResultsFilters(miningResultsUrlParam('runId'));

    if (robotFilter) {
        robotFilter.addEventListener('change', function() {
            applyMiningResultsFilters();
        });
    }
    if (areaFilter) {
        areaFilter.addEventListener('change', function() {
            applyMiningResultsFilters();
        });
    }
    if (sortFilter) {
        sortFilter.addEventListener('change', function() {
            applyMiningResultsSort();
            applyMiningResultsFilters();
        });
    }

    var runCards = document.querySelectorAll('.mining-results-run-card');
    for (var runIndex = 0; runIndex < runCards.length; runIndex += 1) {
        runCards[runIndex].addEventListener('click', function(event) {
            selectMiningResultRun(event.currentTarget.getAttribute('data-run-id'));
        });
    }
})();
</script>"#;
