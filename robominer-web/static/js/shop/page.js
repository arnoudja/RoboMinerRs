(function() {
    var pageRoot = document.querySelector('.shop-page');
    var STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-filter-storage-key') || 'robominer.shop.filterSelections'
        : 'robominer.shop.filterSelections';

    function readStoredShopFilters() {
        return window.RoboMinerSessionStore.readJson(STORAGE_KEY);
    }

    function writeStoredShopFilters() {
        var stored = {};
        var typeSelect = document.getElementById('robotPartTypeId');
        var tierSelect = document.getElementById('tierId');
        if (typeSelect && typeSelect.value) {
            stored.selectedRobotPartTypeId = typeSelect.value;
        }
        if (tierSelect && tierSelect.value) {
            stored.selectedTierId = tierSelect.value;
        }
        var activePanel = document.querySelector('.shop-detail-panel-active:not(.shop-filter-hidden)');
        if (activePanel) {
            var partId = activePanel.getAttribute('data-part-id');
            if (partId) {
                stored.selectedRobotPartId = partId;
            }
        }
        window.RoboMinerSessionStore.writeJson(STORAGE_KEY, stored);
    }

    function urlHasShopFilterParams() {
        return window.RoboMinerUrlQuery.hasAnyParam([
            'selectedRobotPartTypeId',
            'selectedTierId',
            'selectedRobotPartId'
        ]);
    }

    function applyStoredSelectValue(select, value) {
        if (!select || !value || !select.querySelector('option[value="' + value + '"]')) {
            return false;
        }
        if (select.value === value) {
            return false;
        }
        select.value = value;
        return true;
    }

    function collectShopQueryParams() {
        var params = {};
        var typeSelect = document.getElementById('robotPartTypeId');
        var tierSelect = document.getElementById('tierId');
        if (typeSelect && typeSelect.value) {
            params.selectedRobotPartTypeId = typeSelect.value;
        }
        if (tierSelect && tierSelect.value) {
            params.selectedTierId = tierSelect.value;
        }
        var activePanel = document.querySelector('.shop-detail-panel-active');
        if (activePanel) {
            params.selectedRobotPartId = activePanel.getAttribute('data-part-id');
        }
        return params;
    }

    function syncShopUrl() {
        window.RoboMinerUrlQuery.sync('shop', collectShopQueryParams());
        writeStoredShopFilters();
    }

    function matchesFilter(element, typeId, tierId) {
        return element.getAttribute('data-type-id') === typeId
            && element.getAttribute('data-tier-id') === tierId;
    }

    function applyShopFilters() {
        var typeSelect = document.getElementById('robotPartTypeId');
        var tierSelect = document.getElementById('tierId');
        if (!typeSelect || !tierSelect) {
            return;
        }
        var typeId = typeSelect.value;
        var tierId = tierSelect.value;
        var cards = document.querySelectorAll('.shop-part-card-compact');
        var panels = document.querySelectorAll('.shop-detail-panel');
        var firstVisiblePartId = null;
        for (var index = 0; index < cards.length; index += 1) {
            var card = cards[index];
            if (matchesFilter(card, typeId, tierId)) {
                card.classList.remove('shop-filter-hidden');
                if (!firstVisiblePartId) {
                    firstVisiblePartId = card.getAttribute('data-part-id');
                }
            } else {
                card.classList.remove('shop-part-card-active');
                card.classList.add('shop-filter-hidden');
            }
        }
        for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            var panel = panels[panelIndex];
            if (matchesFilter(panel, typeId, tierId)) {
                panel.classList.remove('shop-filter-hidden');
            } else {
                panel.classList.remove('shop-detail-panel-active');
                panel.classList.add('shop-filter-hidden');
            }
        }
        var empty = document.getElementById('shopCatalogEmpty');
        if (empty) {
            empty.hidden = firstVisiblePartId !== null;
        }
        var preferredPartId = shopUrlPartId();
        if (preferredPartId && document.querySelector('.shop-part-card-compact[data-part-id="' + preferredPartId + '"]:not(.shop-filter-hidden)')) {
            selectShopPart(preferredPartId, false);
        } else {
            var activeCard = document.querySelector('.shop-part-card-compact.shop-part-card-active:not(.shop-filter-hidden)');
            if (!activeCard && firstVisiblePartId) {
                selectShopPart(firstVisiblePartId, false);
            }
        }
        syncShopFormState();
        syncShopUrl();
    }

    function shopUrlPartId() {
        return window.RoboMinerUrlQuery.getParam('selectedRobotPartId');
    }

    function syncShopFormState() {
        var typeSelect = document.getElementById('robotPartTypeId');
        var tierSelect = document.getElementById('tierId');
        var activePanel = document.querySelector('.shop-detail-panel-active:not(.shop-filter-hidden)');
        var partId = activePanel ? activePanel.getAttribute('data-part-id') : '';
        var forms = document.querySelectorAll('.shop-action-form');
        for (var formIndex = 0; formIndex < forms.length; formIndex += 1) {
            var form = forms[formIndex];
            if (typeSelect) {
                var typeInput = form.querySelector('input[name="selectedRobotPartTypeId"]');
                if (typeInput) {
                    typeInput.value = typeSelect.value;
                }
            }
            if (tierSelect) {
                var tierInput = form.querySelector('input[name="selectedTierId"]');
                if (tierInput) {
                    tierInput.value = tierSelect.value;
                }
            }
            if (partId) {
                var partInput = form.querySelector('input[name="selectedRobotPartId"]');
                if (partInput) {
                    partInput.value = partId;
                }
            }
        }
    }

    function selectShopPart(partId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        var cards = document.querySelectorAll('.shop-part-card-compact');
        var panels = document.querySelectorAll('.shop-detail-panel');
        for (var index = 0; index < cards.length; index += 1) {
            var card = cards[index];
            if (card.getAttribute('data-part-id') === partId) {
                card.classList.add('shop-part-card-active');
            } else {
                card.classList.remove('shop-part-card-active');
            }
        }
        for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
            var panel = panels[panelIndex];
            if (panel.getAttribute('data-part-id') === partId) {
                panel.classList.add('shop-detail-panel-active');
            } else {
                panel.classList.remove('shop-detail-panel-active');
            }
        }
        syncShopFormState();
        if (updateUrl) {
            syncShopUrl();
        }
    }

    var typeSelect = document.getElementById('robotPartTypeId');
    if (typeSelect) {
        typeSelect.addEventListener('change', applyShopFilters);
    }
    var tierSelect = document.getElementById('tierId');
    if (tierSelect) {
        tierSelect.addEventListener('change', applyShopFilters);
    }

    function restoreShopFiltersFromStorage() {
        if (urlHasShopFilterParams()) {
            writeStoredShopFilters();
            applyShopFilters();
            return;
        }
        var stored = readStoredShopFilters();
        if (!stored) {
            applyShopFilters();
            return;
        }
        var typeSelectForRestore = document.getElementById('robotPartTypeId');
        var tierSelectForRestore = document.getElementById('tierId');
        applyStoredSelectValue(typeSelectForRestore, stored.selectedRobotPartTypeId);
        applyStoredSelectValue(tierSelectForRestore, stored.selectedTierId);
        applyShopFilters();
        if (stored.selectedRobotPartId) {
            var restoredCard = document.querySelector(
                '.shop-part-card-compact[data-part-id="' + stored.selectedRobotPartId + '"]:not(.shop-filter-hidden)'
            );
            if (restoredCard) {
                selectShopPart(stored.selectedRobotPartId, true);
            }
        }
    }

    var cards = document.querySelectorAll('.shop-part-card-compact');
    for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
        cards[cardIndex].addEventListener('click', function(event) {
            selectShopPart(event.currentTarget.getAttribute('data-part-id'));
        });
    }

    restoreShopFiltersFromStorage();
    function confirmShopSell(event) {
        var sellAllInput = event.target.querySelector('input[name="sellAllUnassigned"]');
        if (sellAllInput) {
            var unassignedCount = parseInt(
                event.target.getAttribute('data-unassigned-count') || '0',
                10
            );
            if (!unassignedCount) {
                event.preventDefault();
                return;
            }
            var sellAllMessage = unassignedCount === 1
                ? 'Sell 1 unassigned robot part?'
                : 'Sell all ' + unassignedCount + ' unassigned robot parts?';
            var form = event.target;
            if (form.getAttribute('data-robominer-confirmed') === '1') {
                form.removeAttribute('data-robominer-confirmed');
                return;
            }
            event.preventDefault();
            robominerConfirm(sellAllMessage, function(confirmed) {
                if (!confirmed) {
                    return;
                }
                form.setAttribute('data-robominer-confirmed', '1');
                if (typeof form.requestSubmit === 'function') {
                    form.requestSubmit(event.submitter || undefined);
                } else {
                    form.submit();
                }
            });
            return;
        }

        var sellInput = event.target.querySelector('input[name="sellRobotPartId"]');
        if (!sellInput) {
            return;
        }
        var partName = 'robot part';
        var panel = event.target.closest('.shop-detail-panel');
        if (panel) {
            var panelName = panel.querySelector('.shopPartName');
            if (panelName) {
                partName = panelName.textContent.trim();
            }
        } else {
            var row = event.target.closest('tr');
            if (row) {
                var rowName = row.querySelector('.shop-inventory-name');
                if (rowName) {
                    partName = rowName.textContent.trim();
                }
            }
        }
        var form = event.target;
        if (form.getAttribute('data-robominer-confirmed') === '1') {
            form.removeAttribute('data-robominer-confirmed');
            return;
        }
        event.preventDefault();
        robominerConfirm('Sell 1 unassigned ' + partName + '?', function(confirmed) {
            if (!confirmed) {
                return;
            }
            form.setAttribute('data-robominer-confirmed', '1');
            if (typeof form.requestSubmit === 'function') {
                form.requestSubmit(event.submitter || undefined);
            } else {
                form.submit();
            }
        });
    }

    var actionForms = document.querySelectorAll('.shop-action-form');
    for (var formIndex = 0; formIndex < actionForms.length; formIndex += 1) {
        actionForms[formIndex].addEventListener('submit', confirmShopSell);
    }
})();
