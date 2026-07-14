use std::collections::HashMap;

use super::{
    ENGINE_PART_TYPE_ID, MEMORY_MODULE_PART_TYPE_ID, ORE_SCANNER_PART_TYPE_ID, ShopPageState,
};
use crate::help_pages;
use crate::html::{escape_html, format_period, layout, selected_attr};
use crate::mining_area_atlas::{
    MiningAreaAtlasLinkTarget, mining_area_atlas_url, render_mining_area_atlas_ore_link,
};

pub(super) fn render_shop_page(
    username: String,
    hud: Option<&str>,
    state: &ShopPageState,
) -> String {
    let part_state_map: HashMap<i64, &robominer_db::ShopRobotPartStateRecord> = state
        .part_states
        .iter()
        .map(|state| (state.robot_part_id, state))
        .collect();
    let mut cost_map: HashMap<i64, Vec<&robominer_db::ShopRobotPartCostRecord>> = HashMap::new();
    for cost in &state.costs {
        cost_map.entry(cost.robot_part_id).or_default().push(cost);
    }
    for costs in cost_map.values_mut() {
        costs.sort_by_key(|cost| std::cmp::Reverse(cost.ore_id));
    }
    let ore_amount_map: HashMap<i64, i32> = state
        .ore_assets
        .iter()
        .map(|asset| (asset.ore_id, asset.amount))
        .collect();

    let filter_storage_key = format!(
        "robominer.shop.filterSelections.{}",
        username.replace([' ', '"', '\''], "_")
    );
    let mut body = String::from(&format!(
        r#"<div class="shop-page" data-filter-storage-key="{}">"#,
        escape_html(&filter_storage_key)
    ));
    render_shop_wallet_strip(&mut body, state);
    render_shop_message(&mut body, state);
    render_shop_filters(&mut body, state);

    body.push_str(r#"<div class="shop-deck">"#);
    body.push_str(r#"<section class="shop-catalog" aria-labelledby="shop-catalog-title">"#);
    body.push_str(r#"<h2 id="shop-catalog-title" class="shop-section-title">Catalog</h2>"#);
    body.push_str(r#"<div class="shop-part-cards">"#);

    let mut part_counters: HashMap<(i64, i64), i32> = HashMap::new();
    let mut matching_parts = 0;
    for part in &state.parts {
        let counter = part_counters
            .entry((part.type_id, part.tier_id))
            .or_insert(0);
        *counter += 1;
        if part.type_id == state.selected_part_type_id && part.tier_id == state.selected_tier_id {
            matching_parts += 1;
        }
        let empty_state;
        let part_state = if let Some(part_state) = part_state_map.get(&part.robot_part_id) {
            *part_state
        } else {
            empty_state = robominer_db::ShopRobotPartStateRecord {
                robot_part_id: part.robot_part_id,
                total_owned: 0,
                assigned: 0,
                unassigned: 0,
                can_buy: false,
                can_sell: false,
            };
            &empty_state
        };
        render_shop_part_compact_card(
            &mut body,
            part,
            part_state,
            cost_map
                .get(&part.robot_part_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &ore_amount_map,
            *counter,
            state,
        );
    }

    body.push_str(&format!(
        r#"<p id="shopCatalogEmpty" class="shop-empty shop-catalog-empty"{}>No parts match this category and quality.</p>"#,
        if matching_parts == 0 { "" } else { " hidden" }
    ));
    body.push_str("</div></section>");

    body.push_str(r#"<aside class="shop-detail" aria-labelledby="shop-detail-title">"#);
    body.push_str(r#"<h2 id="shop-detail-title" class="shop-section-title">Part details</h2>"#);
    body.push_str(r#"<div class="shop-detail-panels">"#);
    for part in &state.parts {
        let empty_state;
        let part_state = if let Some(part_state) = part_state_map.get(&part.robot_part_id) {
            *part_state
        } else {
            empty_state = robominer_db::ShopRobotPartStateRecord {
                robot_part_id: part.robot_part_id,
                total_owned: 0,
                assigned: 0,
                unassigned: 0,
                can_buy: false,
                can_sell: false,
            };
            &empty_state
        };
        render_shop_part_detail_panel(
            &mut body,
            part,
            part_state,
            cost_map
                .get(&part.robot_part_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &ore_amount_map,
            state,
        );
    }
    body.push_str("</div></aside></div>");

    render_shop_inventory(&mut body, state, &part_state_map);
    body.push_str(
        r#"<script>
(function() {
    var pageRoot = document.querySelector('.shop-page');
    var STORAGE_KEY = pageRoot
        ? pageRoot.getAttribute('data-filter-storage-key') || 'robominer.shop.filterSelections'
        : 'robominer.shop.filterSelections';

    function readStoredShopFilters() {
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

    function writeStoredShopFilters() {
        try {
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
            window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(stored));
        } catch (error) {
        }
    }

    function urlHasShopFilterParams() {
        var search = window.location.search;
        if (!search) {
            return false;
        }
        var params = search.substring(1).split('&');
        for (var paramIndex = 0; paramIndex < params.length; paramIndex += 1) {
            var name = decodeURIComponent(params[paramIndex].split('=')[0]);
            if (name === 'selectedRobotPartTypeId'
                || name === 'selectedTierId'
                || name === 'selectedRobotPartId') {
                return true;
            }
        }
        return false;
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
        var params = [];
        var typeSelect = document.getElementById('robotPartTypeId');
        var tierSelect = document.getElementById('tierId');
        if (typeSelect && typeSelect.value) {
            params.push(encodeURIComponent('selectedRobotPartTypeId') + '=' + encodeURIComponent(typeSelect.value));
        }
        if (tierSelect && tierSelect.value) {
            params.push(encodeURIComponent('selectedTierId') + '=' + encodeURIComponent(tierSelect.value));
        }
        var activePanel = document.querySelector('.shop-detail-panel-active');
        if (activePanel) {
            params.push(encodeURIComponent('selectedRobotPartId') + '=' + encodeURIComponent(activePanel.getAttribute('data-part-id')));
        }
        return params.join('&');
    }

    function syncShopUrl() {
        var query = collectShopQueryParams();
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', query ? 'shop?' + query : 'shop');
        }
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
        var search = window.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === 'selectedRobotPartId' && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
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
</script>"#,
    );
    body.push_str("</div>");

    layout("RoboMiner - Shop", "shop", &username, hud, &body)
}

fn render_shop_wallet_strip(body: &mut String, state: &ShopPageState) {
    body.push_str(r#"<section class="shop-wallet" aria-label="Wallet">"#);
    body.push_str(r#"<div class="shop-wallet-heading">"#);
    body.push_str(r#"<h1 class="shop-page-title">Parts shop</h1>"#);
    body.push_str("</div>");

    if state.ore_assets.is_empty() {
        body.push_str(r#"<p class="shop-wallet-empty">No ore in wallet yet.</p>"#);
    } else {
        body.push_str(r#"<ul class="shop-wallet-list">"#);
        for asset in &state.ore_assets {
            let balance_class = if asset.amount >= asset.max_allowed {
                "shop-wallet-full"
            } else {
                "shop-wallet-ok"
            };
            body.push_str(&format!(
                r#"<li class="shop-wallet-item {balance_class}"><div class="shop-wallet-item-row"><span class="shop-wallet-ore">{}</span><span class="shop-wallet-amount">{}/{}</span></div>{}</li>"#,
                escape_html(&asset.ore_name),
                asset.amount,
                asset.max_allowed,
                render_mining_area_atlas_ore_link(
                    asset.ore_id,
                    &asset.ore_name,
                    MiningAreaAtlasLinkTarget::StandalonePage,
                    "shop-atlas-link",
                ),
            ));
        }
        body.push_str("</ul>");
    }

    body.push_str("</section>");
}

fn render_shop_message(body: &mut String, state: &ShopPageState) {
    let Some(message) = &state.message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "shop-banner shop-banner-error"
    } else {
        "shop-banner shop-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}

fn render_shop_filters(body: &mut String, state: &ShopPageState) {
    body.push_str(r#"<section class="shop-filters" aria-label="Catalog filters">"#);
    body.push_str(r#"<div class="shop-filter-form">"#);
    body.push_str(r#"<label class="shop-filter-label" for="robotPartTypeId">Category <select id="robotPartTypeId" name="selectedRobotPartTypeId" class="tableitem shop-filter-select">"#);
    for part_type in &state.part_types {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            part_type.id,
            selected_attr(part_type.id == state.selected_part_type_id),
            escape_html(&part_type.type_name)
        ));
    }
    body.push_str("</select></label>");
    body.push_str(r#"<label class="shop-filter-label" for="tierId">Quality <select id="tierId" name="selectedTierId" class="tableitem shop-filter-select">"#);
    for ore in &state.ores {
        body.push_str(&format!(
            r#"<option value="{}"{}>{} quality</option>"#,
            ore.id,
            selected_attr(ore.id == state.selected_tier_id),
            escape_html(&ore.ore_name)
        ));
    }
    body.push_str("</select></label></div>");
    body.push_str(&format!(
        r#"<p class="shop-atlas-helper">Need ore for parts? <a class="shop-atlas-link" href="{}">Compare all areas</a>.</p>"#,
        escape_html(&mining_area_atlas_url(
            MiningAreaAtlasLinkTarget::StandalonePage,
            None,
            false,
        )),
    ));
    body.push_str(&help_pages::render_page_help_hint(
        "What do capacity, power, and weight stats mean?",
        "helpMechanics",
        "Read the mechanics guide",
    ));
    body.push_str("</section>");
}

fn render_shop_inventory(
    body: &mut String,
    state: &ShopPageState,
    part_state_map: &HashMap<i64, &robominer_db::ShopRobotPartStateRecord>,
) {
    body.push_str(r#"<section class="shop-inventory" aria-labelledby="shop-inventory-title">"#);
    body.push_str(r#"<div class="shop-inventory-header">"#);
    body.push_str(r#"<h2 id="shop-inventory-title" class="shop-section-title">Owned items</h2>"#);
    render_shop_sell_all_action(body, state);
    body.push_str("</div>");
    body.push_str(r#"<table class="shop-inventory-table"><thead><tr><th>Item name</th><th>Quality</th><th>Amount</th><th>Unassigned</th><th></th></tr></thead><tbody>"#);

    let mut owned_rows: Vec<(
        &robominer_db::ShopRobotPartCatalogRecord,
        &robominer_db::ShopRobotPartStateRecord,
    )> = state
        .parts
        .iter()
        .filter_map(|part| {
            part_state_map
                .get(&part.robot_part_id)
                .filter(|part_state| part_state.total_owned > 0)
                .map(|part_state| (part, *part_state))
        })
        .collect();
    owned_rows.sort_by(|(left_part, left_state), (right_part, right_state)| {
        right_state
            .unassigned
            .cmp(&left_state.unassigned)
            .then_with(|| left_part.part_name.cmp(&right_part.part_name))
            .then_with(|| left_part.tier_name.cmp(&right_part.tier_name))
    });

    if owned_rows.is_empty() {
        body.push_str(
            r#"<tr><td colspan="5" class="shop-empty">You do not own any robot parts yet.</td></tr>"#,
        );
    } else {
        for (part, part_state) in owned_rows {
            body.push_str(&format!(
                r#"<tr><td class="shop-inventory-name">{}</td><td>{} quality</td><td>{}</td><td>{}</td><td class="shop-inventory-action">{}</td></tr>"#,
                escape_html(&part.part_name),
                escape_html(&part.tier_name),
                part_state.total_owned,
                part_state.unassigned,
                render_shop_sell_action(part, part_state, state)
            ));
        }
    }

    body.push_str("</tbody></table>");
    body.push_str("</section>");
}

fn shop_total_unassigned(part_states: &[robominer_db::ShopRobotPartStateRecord]) -> i32 {
    part_states.iter().map(|state| state.unassigned).sum()
}

fn render_shop_sell_all_action(body: &mut String, state: &ShopPageState) {
    let unassigned_total = shop_total_unassigned(&state.part_states);
    let enabled = unassigned_total > 0;
    let disabled_attr = if enabled { "" } else { " disabled" };
    let title_attr = if enabled {
        String::new()
    } else {
        r#" title="No unassigned robot parts to sell.""#.to_string()
    };

    body.push_str(&format!(
        r#"<div class="shop-inventory-actions"><form action="shop" method="post" class="shop-action-form shop-sell-all-form" data-unassigned-count="{unassigned_total}"><input type="hidden" name="sellAllUnassigned" value="1"/><input type="hidden" name="selectedRobotPartTypeId" value="{}"/><input type="hidden" name="selectedTierId" value="{}"/><input type="hidden" name="selectedRobotPartId" value="{}"/><button type="submit" class="shop-btn shop-btn-danger"{disabled_attr}{title_attr}>Sell all unassigned</button></form></div>"#,
        state.selected_part_type_id,
        state.selected_tier_id,
        state.selected_part_id,
    ));
}

fn render_shop_part_compact_card(
    body: &mut String,
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    part_number: i32,
    page_state: &ShopPageState,
) {
    let active_class = if part.robot_part_id == page_state.selected_part_id {
        " shop-part-card-active"
    } else {
        ""
    };
    let filter_hidden = if part.type_id == page_state.selected_part_type_id
        && part.tier_id == page_state.selected_tier_id
    {
        ""
    } else {
        " shop-filter-hidden"
    };

    body.push_str(&format!(
        r#"<button type="button" class="shop-part-card shop shop-part-card-compact{active_class}{filter_hidden}" id="robotPartTypeRow{}_{}_{}" data-part-id="{}" data-type-id="{}" data-tier-id="{}" data-can-buy="{}" data-cost-total="{}">"#,
        part.type_id,
        part.tier_id,
        part_number,
        part.robot_part_id,
        part.type_id,
        part.tier_id,
        if state.can_buy { "1" } else { "0" },
        shop_total_cost(costs),
    ));
    body.push_str(&format!(
        r#"<span class="shop-part-heading"><span class="shop-part-name shopPartName">{}</span><span class="shop-part-tier">{} quality</span></span>"#,
        escape_html(&part.part_name),
        escape_html(&part.tier_name)
    ));
    body.push_str(r#"<span class="shop-part-highlights">"#);
    push_shop_highlight(body, "Ore", part.ore_capacity, " units");
    push_shop_highlight(body, "Mining", part.mining_capacity, " o/c");
    push_shop_highlight(body, "Battery", part.battery_capacity, " pc");
    push_shop_highlight(body, "CPU", part.cpu_capacity, " i/c");
    if part.type_id == ORE_SCANNER_PART_TYPE_ID {
        push_shop_highlight(body, "Scan", part.scan_distance, "");
    } else if part.type_id == MEMORY_MODULE_PART_TYPE_ID {
        push_shop_highlight(body, "Memory", part.memory_capacity, "");
    } else if part.type_id == ENGINE_PART_TYPE_ID {
        push_shop_highlight(body, "Forward", part.forward_capacity, "");
    } else if part.scan_time > 0 {
        push_shop_highlight(body, "Scan", part.scan_time, " cyc");
    }
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="shop-part-cost-summary">{}</span>"#,
        escape_html(&shop_cost_summary(costs, ore_amount_map))
    ));
    if state.total_owned > 0 {
        body.push_str(&format!(
            r#"<span class="shop-part-owned-badge">Owned: {}</span>"#,
            state.total_owned
        ));
    }
    body.push_str("</button>");
}

fn render_shop_part_detail_panel(
    body: &mut String,
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    page_state: &ShopPageState,
) {
    let active_class = if part.robot_part_id == page_state.selected_part_id {
        " shop-detail-panel-active"
    } else {
        ""
    };
    let filter_hidden = if part.type_id == page_state.selected_part_type_id
        && part.tier_id == page_state.selected_tier_id
    {
        ""
    } else {
        " shop-filter-hidden"
    };

    body.push_str(&format!(
        r#"<div class="shop-detail-panel{active_class}{filter_hidden}" id="shopPartDetails{}" data-part-id="{}" data-type-id="{}" data-tier-id="{}"><header class="shop-detail-header"><div><h3 class="shop-part-name shopPartName">{}</h3><p class="shop-part-tier">{} quality</p></div><div class="shop-part-buy shopFirstRow">{}</div></header>"#,
        part.robot_part_id,
        part.robot_part_id,
        part.type_id,
        part.tier_id,
        escape_html(&part.part_name),
        escape_html(&part.tier_name),
        render_shop_buy_action(part, state, costs, ore_amount_map, page_state),
    ));

    body.push_str(r#"<dl class="shop-part-stats">"#);
    add_shop_stat_entry(body, "Ore capacity:", part.ore_capacity, " units");
    add_shop_stat_entry(body, "Mining capacity:", part.mining_capacity, " o/c");
    add_shop_stat_entry(body, "Battery capacity:", part.battery_capacity, " pc");
    add_shop_stat_entry(body, "Memory size:", part.memory_capacity, "");
    add_shop_stat_entry(body, "CPU speed:", part.cpu_capacity, " i/c");
    if part.forward_capacity > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>Engine power</dt><dd>{} forward, {} backward, {} rotate</dd></div>"#,
            part.forward_capacity, part.backward_capacity, part.rotate_capacity
        ));
    }
    if part.scan_time > 0 {
        add_shop_stat_entry(body, "Scan time:", part.scan_time, " cycles");
        add_shop_stat_entry(body, "Scan distance:", part.scan_distance, "");
    }
    add_shop_stat_entry(body, "Power consumption:", part.power_usage, " p");
    if part.recharge_time > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>Recharge time</dt><dd>{}</dd></div>"#,
            format_period(part.recharge_time)
        ));
    }
    add_shop_stat_entry(body, "Weight:", part.weight, "");
    add_shop_stat_entry(body, "Volume:", part.volume, "");
    body.push_str("</dl>");

    if state.total_owned > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-owned"><p class="shop-part-owned-summary"><span class="important">Owned:</span> {} total, {} unassigned</p><div class="shop-part-owned-action">{}</div></div>"#,
            state.total_owned,
            state.unassigned,
            render_shop_sell_action(part, state, page_state),
        ));
    }

    body.push_str(r#"<div class="shop-part-costs"><p class="shop-part-costs-title">Ore cost</p><ul class="shop-part-cost-list">"#);
    for cost in costs {
        let user_amount = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        body.push_str(&format!(
            r#"<li><span>{}: {}</span><span class="{}">({})</span>{}</li>"#,
            escape_html(&cost.ore_name),
            cost.amount,
            if user_amount >= cost.amount {
                "sufficientbalance"
            } else {
                "insufficientbalance"
            },
            user_amount,
            render_mining_area_atlas_ore_link(
                cost.ore_id,
                &cost.ore_name,
                MiningAreaAtlasLinkTarget::StandalonePage,
                "shop-atlas-link",
            ),
        ));
    }
    body.push_str("</ul></div></div>");
}

fn render_shop_buy_action(
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
    page_state: &ShopPageState,
) -> String {
    let block_reason = shop_buy_block_reason(state, costs, ore_amount_map);
    let mut html = shop_button(
        "Buy part",
        "buyRobotPartId",
        part.robot_part_id,
        page_state,
        state.can_buy,
        ShopButtonStyle::Primary,
        block_reason.as_deref(),
    );
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="shop-action-hint">{}</p>"#,
            escape_html(&reason)
        ));
        if let Some(shortfall) = costs
            .iter()
            .find(|cost| ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0) < cost.amount)
        {
            html.push_str(&render_mining_area_atlas_ore_link(
                shortfall.ore_id,
                &shortfall.ore_name,
                MiningAreaAtlasLinkTarget::StandalonePage,
                "shop-atlas-link shop-action-atlas-link",
            ));
        }
    }
    html
}

fn render_shop_sell_action(
    part: &robominer_db::ShopRobotPartCatalogRecord,
    state: &robominer_db::ShopRobotPartStateRecord,
    page_state: &ShopPageState,
) -> String {
    let block_reason = shop_sell_block_reason(state);
    let mut html = shop_button(
        "Sell unassigned",
        "sellRobotPartId",
        part.robot_part_id,
        page_state,
        state.can_sell,
        ShopButtonStyle::Danger,
        block_reason.as_deref(),
    );
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="shop-action-hint">{}</p>"#,
            escape_html(&reason)
        ));
    }
    html
}

fn shop_buy_block_reason(
    state: &robominer_db::ShopRobotPartStateRecord,
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> Option<String> {
    if state.can_buy {
        return None;
    }
    for cost in costs {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return Some(format!("Need {} more {}.", need, cost.ore_name));
        }
    }
    if state.total_owned > 0 {
        Some("You already own enough of this part for your robots.".to_string())
    } else {
        Some("You need a robot before buying parts.".to_string())
    }
}

fn shop_sell_block_reason(state: &robominer_db::ShopRobotPartStateRecord) -> Option<String> {
    if state.can_sell {
        return None;
    }
    if state.total_owned > 0 {
        Some("All units are assigned to robots.".to_string())
    } else {
        Some("You do not own this part.".to_string())
    }
}

fn shop_total_cost(costs: &[&robominer_db::ShopRobotPartCostRecord]) -> i32 {
    costs.iter().map(|cost| cost.amount).sum()
}

fn shop_cost_summary(
    costs: &[&robominer_db::ShopRobotPartCostRecord],
    ore_amount_map: &HashMap<i64, i32>,
) -> String {
    if costs.is_empty() {
        return "No ore cost".to_string();
    }
    for cost in costs {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return format!("Need {} more {}.", need, cost.ore_name);
        }
    }
    let first = costs[0];
    format!("{} {} — affordable", first.amount, first.ore_name)
}

fn push_shop_highlight(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<span class="shop-part-highlight"><span class="shop-part-highlight-label">{label}</span><span class="shop-part-highlight-value">{value}{suffix}</span></span>"#,
        ));
    }
}

fn add_shop_stat_entry(body: &mut String, label: &str, value: i32, suffix: &str) {
    if value > 0 {
        body.push_str(&format!(
            r#"<div class="shop-part-stat"><dt>{label}</dt><dd>{value}{suffix}</dd></div>"#,
        ));
    }
}

enum ShopButtonStyle {
    Primary,
    Danger,
}

fn shop_button(
    label: &str,
    transaction_field_name: &str,
    robot_part_id: i64,
    page_state: &ShopPageState,
    enabled: bool,
    style: ShopButtonStyle,
    block_reason: Option<&str>,
) -> String {
    let disabled_attr = if enabled { "" } else { " disabled" };
    let title_attr = block_reason
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    let class_name = match style {
        ShopButtonStyle::Primary => "shop-btn shop-btn-primary",
        ShopButtonStyle::Danger => "shop-btn shop-btn-danger",
    };
    format!(
        r#"<form action="shop" method="post" class="shop-action-form"><input type="hidden" name="{}" value="{}"/><input type="hidden" name="selectedRobotPartTypeId" value="{}"/><input type="hidden" name="selectedTierId" value="{}"/><input type="hidden" name="selectedRobotPartId" value="{}"/><button type="submit" class="{class_name}"{disabled_attr}{title_attr}>{}</button></form>"#,
        transaction_field_name,
        robot_part_id,
        page_state.selected_part_type_id,
        page_state.selected_tier_id,
        page_state.selected_part_id,
        label
    )
}
