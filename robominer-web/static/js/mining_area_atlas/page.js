(function() {
    function collectMiningAreaAtlasQueryParams() {
        var params = {};
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreSelect = document.getElementById('miningAreaAtlasOreSort');
        var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
        if (sortSelect && sortSelect.value) {
            params.sort = sortSelect.value;
        }
        if (sortSelect && sortSelect.value === 'ore' && oreSelect && oreSelect.value) {
            params.oreId = oreSelect.value;
        }
        if (affordableOnly && affordableOnly.checked) {
            params.affordable = '1';
        }
        return params;
    }

    function syncMiningAreaAtlasUrl() {
        window.RoboMinerUrlQuery.sync('miningAreaOverview', collectMiningAreaAtlasQueryParams());
    }

    function compareAtlasRows(left, right, sortBy, oreId) {
        if (sortBy === 'name') {
            return left.getAttribute('data-area-name').localeCompare(right.getAttribute('data-area-name'));
        }
        if (sortBy === 'level') {
            var leftLevel = Number(left.getAttribute('data-area-id')) || 0;
            var rightLevel = Number(right.getAttribute('data-area-id')) || 0;
            return rightLevel - leftLevel;
        }
        if (sortBy === 'ore' && oreId) {
            var leftYield = Number(left.getAttribute('data-ore-yield-' + oreId)) || 0;
            var rightYield = Number(right.getAttribute('data-ore-yield-' + oreId)) || 0;
            return rightYield - leftYield;
        }
        var leftTotal = Number(left.getAttribute('data-total-yield')) || 0;
        var rightTotal = Number(right.getAttribute('data-total-yield')) || 0;
        return rightTotal - leftTotal;
    }

    function updateOreSortVisibility() {
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreField = document.getElementById('miningAreaAtlasOreField');
        if (!sortSelect || !oreField) {
            return;
        }
        oreField.hidden = sortSelect.value !== 'ore';
    }

    function applyMiningAreaAtlasControls() {
        var sortSelect = document.getElementById('miningAreaAtlasSort');
        var oreSelect = document.getElementById('miningAreaAtlasOreSort');
        var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
        var tbody = document.getElementById('miningAreaAtlasRows');
        if (!sortSelect || !tbody) {
            return;
        }
        updateOreSortVisibility();
        var sortBy = sortSelect.value || 'level';
        var oreId = oreSelect ? oreSelect.value : '';
        var rows = Array.prototype.slice.call(tbody.querySelectorAll('.mining-area-atlas-row'));
        rows.sort(function(left, right) {
            return compareAtlasRows(left, right, sortBy, oreId);
        });
        for (var rowIndex = 0; rowIndex < rows.length; rowIndex += 1) {
            tbody.appendChild(rows[rowIndex]);
        }
        var visibleCount = 0;
        for (var filterIndex = 0; filterIndex < rows.length; filterIndex += 1) {
            var row = rows[filterIndex];
            var hide = affordableOnly && affordableOnly.checked && row.getAttribute('data-affordable') !== '1';
            row.classList.toggle('mining-area-atlas-filter-hidden', hide);
            if (!hide) {
                visibleCount += 1;
            }
        }
        var empty = document.getElementById('miningAreaAtlasFilterEmpty');
        if (empty) {
            empty.hidden = visibleCount > 0;
        }
        syncMiningAreaAtlasUrl();
    }

    var sortSelect = document.getElementById('miningAreaAtlasSort');
    var oreSelect = document.getElementById('miningAreaAtlasOreSort');
    var affordableOnly = document.getElementById('miningAreaAtlasAffordableOnly');
    if (sortSelect) {
        var preferredSort = window.RoboMinerUrlQuery.getParam('sort');
        if (preferredSort) {
            for (var sortIndex = 0; sortIndex < sortSelect.options.length; sortIndex += 1) {
                if (sortSelect.options[sortIndex].value === preferredSort) {
                    sortSelect.value = preferredSort;
                    break;
                }
            }
        }
    }
    if (oreSelect) {
        var preferredOreId = window.RoboMinerUrlQuery.getParam('oreId');
        if (preferredOreId) {
            for (var oreIndex = 0; oreIndex < oreSelect.options.length; oreIndex += 1) {
                if (oreSelect.options[oreIndex].value === preferredOreId) {
                    oreSelect.value = preferredOreId;
                    break;
                }
            }
        }
    }
    if (affordableOnly) {
        affordableOnly.checked = window.RoboMinerUrlQuery.getParam('affordable') === '1';
    }
    applyMiningAreaAtlasControls();
    if (sortSelect) {
        sortSelect.addEventListener('change', applyMiningAreaAtlasControls);
    }
    if (oreSelect) {
        oreSelect.addEventListener('change', applyMiningAreaAtlasControls);
    }
    if (affordableOnly) {
        affordableOnly.addEventListener('change', applyMiningAreaAtlasControls);
    }
})();
