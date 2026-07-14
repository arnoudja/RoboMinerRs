use std::collections::HashMap;

use crate::help_pages;
use crate::html::{escape_html, format_period, layout, selected_attr};
use crate::mining_queue_page::{MiningQueueDisplayItem, MiningQueuePageState};

use super::mining_queue_status_description;

pub(super) fn render_mining_queue_page(
    username: String,
    hud: Option<&str>,
    state: &MiningQueuePageState,
) -> String {
    let mut item_map: HashMap<i64, Vec<&MiningQueueDisplayItem>> = HashMap::new();
    for item in &state.items {
        item_map.entry(item.robot_id).or_default().push(item);
    }
    let mut cost_map: HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaCostRecord>> =
        HashMap::new();
    for cost in &state.costs {
        cost_map.entry(cost.mining_area_id).or_default().push(cost);
    }
    let mut supply_map: HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaSupplyRecord>> =
        HashMap::new();
    for supply in &state.supplies {
        supply_map
            .entry(supply.mining_area_id)
            .or_default()
            .push(supply);
    }
    let mut yield_map: HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaYieldRecord>> =
        HashMap::new();
    for area_yield in &state.yields {
        yield_map
            .entry(area_yield.mining_area_id)
            .or_default()
            .push(area_yield);
    }
    let mut score_map: HashMap<(i64, i64), f64> = HashMap::new();
    for score in &state.scores {
        score_map.insert((score.robot_id, score.mining_area_id), score.score);
    }
    let ore_amount_map: HashMap<i64, i32> = state
        .ore_assets
        .iter()
        .map(|asset| (asset.ore_id, asset.amount))
        .collect();

    let mut area_map: HashMap<i64, &robominer_db::MiningQueuePageAreaRecord> = HashMap::new();
    for area in &state.areas {
        area_map.insert(area.mining_area_id, area);
    }

    let area_storage_key = format!(
        "robominer.miningQueue.areaSelections.{}",
        username.replace([' ', '"', '\''], "_")
    );
    let mut body = String::from(&format!(
        r#"<div class="mining-queue-page" data-area-storage-key="{}">"#,
        escape_html(&area_storage_key)
    ));
    render_wallet_strip(&mut body, state);
    if !state.robots.is_empty() && state.items.is_empty() {
        body.push_str(&help_pages::render_page_help_hint(
            "Getting started?",
            "helpTutorial?step=1",
            "Follow the step-by-step tutorial",
        ));
    }
    if let Some(error_message) = &state.error_message {
        body.push_str(&format!(
            r#"<p class="error mining-queue-error">{}</p>"#,
            escape_html(error_message)
        ));
    }

    body.push_str(r#"<div class="mining-queue-deck">"#);
    body.push_str(r#"<div class="mining-queue-robots">"#);
    if state.robots.is_empty() {
        body.push_str(
            r#"<p class="mining-queue-empty mining-queue-no-robots">No robots yet. <a href="shop">Visit the shop</a> to buy your first robot.</p>"#,
        );
    } else {
        for robot in &state.robots {
            let queue_items = item_map
                .get(&robot.robot_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            render_robot_card(
                &mut body,
                state,
                robot,
                queue_items,
                &cost_map,
                &ore_amount_map,
                &area_map,
            );
        }
    }
    body.push_str("</div>");

    body.push_str(r#"<div class="miningqueue mining-queue-inspector">"#);
    body.push_str(
        r#"<h1 class="mining-queue-inspector-title">Mining area info</h1><a class="buttonlink mining-queue-overview-link" href="miningAreaOverview">Compare all areas</a>"#,
    );
    body.push_str(r#"<div class="mining-queue-inspector-header"><label class="mining-queue-inspector-label" for="infoMiningAreaId">Mining area <select id="infoMiningAreaId" name="infoMiningAreaId" class="tableitem mining-queue-inspector-select">"#);
    for area in &state.areas {
        body.push_str(&format!(
            r#"<option value="{}"{}>{}</option>"#,
            area.mining_area_id,
            selected_attr(area.mining_area_id == state.selected_info_area_id),
            escape_html(&area.area_name)
        ));
    }
    body.push_str("</select></label></div>");
    body.push_str(r#"<table class="mining-queue-inspector-table">"#);

    for area in &state.areas {
        render_mining_area_details(
            &mut body,
            area,
            cost_map
                .get(&area.mining_area_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            supply_map
                .get(&area.mining_area_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            yield_map
                .get(&area.mining_area_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &state.robots,
            &score_map,
            &ore_amount_map,
            area.mining_area_id == state.selected_info_area_id,
        );
    }

    body.push_str("</table></div></div>");
    body.push_str(
        r#"<script>
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
</script>"#,
    );
    body.push_str("</div>");

    layout(
        "RoboMiner - Mining queue",
        "miningQueue",
        &username,
        hud,
        &body,
    )
}

fn render_wallet_strip(body: &mut String, state: &MiningQueuePageState) {
    body.push_str(r#"<section class="mining-queue-wallet" aria-label="Wallet and queue limits">"#);
    body.push_str(r#"<div class="mining-queue-wallet-heading">"#);
    body.push_str(r#"<h1 class="mining-queue-page-title">Mining queue</h1>"#);
    body.push_str("</div>");

    body.push_str(&crate::html::render_claimed_ore_rewards_banner(
        "mining-queue-claim-banner",
        &state.claimed_results,
        true,
    ));

    if state.ore_assets.is_empty() {
        body.push_str(r#"<p class="mining-queue-wallet-empty">No ore in wallet yet.</p>"#);
    } else {
        body.push_str(r#"<ul class="mining-queue-wallet-list">"#);
        for asset in &state.ore_assets {
            let balance_class = if asset.amount >= asset.max_allowed {
                "mining-queue-wallet-full"
            } else {
                "mining-queue-wallet-ok"
            };
            body.push_str(&format!(
                r#"<li class="mining-queue-wallet-item {balance_class}"><span class="mining-queue-wallet-ore">{}</span><span class="mining-queue-wallet-amount">{}/{}</span></li>"#,
                escape_html(&asset.ore_name),
                asset.amount,
                asset.max_allowed
            ));
        }
        body.push_str("</ul>");
    }

    body.push_str("</section>");
}

fn render_robot_card(
    body: &mut String,
    state: &MiningQueuePageState,
    robot: &robominer_db::MiningQueuePageRobotRecord,
    queue_items: &[&MiningQueueDisplayItem],
    cost_map: &HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaCostRecord>>,
    ore_amount_map: &HashMap<i64, i32>,
    area_map: &HashMap<i64, &robominer_db::MiningQueuePageAreaRecord>,
) {
    let queue_limit = i64::from(state.asset_summary.mining_queue_size);
    let selected_area_id = state
        .selected_robot_area_ids
        .get(&robot.robot_id)
        .copied()
        .unwrap_or(0);
    let selected_enqueue_block_reason = enqueue_block_reason(
        state,
        queue_items.len(),
        selected_area_id,
        cost_map,
        ore_amount_map,
    );
    let can_enqueue = selected_enqueue_block_reason.is_none();
    let disabled_attr = if can_enqueue { "" } else { " disabled" };
    let title_attr = selected_enqueue_block_reason
        .as_ref()
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();

    body.push_str(&format!(
        r#"<form action="miningQueue" method="post" class="mining-queue-card"><input type="hidden" name="robotId" value="{}"/>"#,
        robot.robot_id
    ));
    render_mining_queue_selection_state_inputs(body, state, Some(robot.robot_id));

    body.push_str(r#"<div class="mining-queue-card-header">"#);
    body.push_str(&format!(
        r#"<h2 class="mining-queue-robot-name"><a href="robot?robotId={}">{}</a></h2>"#,
        robot.robot_id,
        escape_html(&robot.robot_name)
    ));
    body.push_str(&format!(
        r#"<p class="mining-queue-slot-count">{}/{} slots</p>"#,
        queue_items.len(),
        queue_limit
    ));
    body.push_str("</div>");

    if queue_items.is_empty() {
        body.push_str(
            r#"<p class="mining-queue-empty">Queue empty — choose an area below and add a run.</p>"#,
        );
    } else if let Some(active_item) = queue_items.first() {
        body.push_str(r#"<div class="mining-queue-active">"#);
        body.push_str(r#"<p class="mining-queue-section-label">Current run</p>"#);
        body.push_str(r#"<div class="mining-queue-run mining-queue-run-active">"#);
        let progress_total = active_run_progress_total(active_item, robot, area_map);
        render_queue_run_row(body, active_item, false, true, progress_total);
        if let Some(total_seconds) = progress_total {
            render_run_progress(body, active_item.time_left_seconds, total_seconds);
        }
        body.push_str("</div></div>");
    }

    if queue_items.len() > 1 {
        body.push_str(r#"<div class="mining-queue-upcoming">"#);
        body.push_str(r#"<p class="mining-queue-section-label">Queued</p>"#);
        body.push_str(r#"<ul class="mining-queue-upcoming-list">"#);
        for item in &queue_items[1..] {
            body.push_str("<li>");
            render_queue_run_row(body, item, true, false, None);
            body.push_str("</li>");
        }
        body.push_str("</ul></div>");
    }

    body.push_str(r#"<div class="mining-queue-actions">"#);
    body.push_str(&format!(
        r#"<label class="mining-queue-area-label" for="miningArea{robot_id}">Area <select id="miningArea{robot_id}" name="miningArea{robot_id}" class="tableitem mining-queue-area-select">"#,
        robot_id = robot.robot_id
    ));
    for area in &state.areas {
        let area_block_reason = enqueue_block_reason(
            state,
            queue_items.len(),
            area.mining_area_id,
            cost_map,
            ore_amount_map,
        );
        let block_reason_attr = area_block_reason
            .as_ref()
            .map(|reason| format!(r#" data-block-reason="{}""#, escape_html(reason)))
            .unwrap_or_default();
        body.push_str(&format!(
            r#"<option value="{}"{}{}>{}</option>"#,
            area.mining_area_id,
            selected_attr(area.mining_area_id == selected_area_id),
            block_reason_attr,
            escape_html(&area.area_name)
        ));
    }
    body.push_str("</select></label>");

    body.push_str(r#"<div class="mining-queue-action-buttons">"#);
    body.push_str(&format!(
        r#"<button type="submit" class="mining-queue-btn mining-queue-btn-primary" name="submitType" value="add"{disabled_attr}{title_attr}>Add to queue</button>"#
    ));
    body.push_str(&format!(
        r#"<button type="submit" class="mining-queue-btn" name="submitType" value="fill"{disabled_attr}{title_attr}>Fill queue</button>"#
    ));
    body.push_str(
        r#"<p class="mining-queue-action-help">Fill queue adds runs until this robot's slots are full.</p>"#,
    );
    let hint_hidden = if selected_enqueue_block_reason.is_some() {
        ""
    } else {
        " hidden"
    };
    body.push_str(&format!(
        r#"<p class="mining-queue-action-hint"{hint_hidden}>{}</p>"#,
        escape_html(selected_enqueue_block_reason.as_deref().unwrap_or(""))
    ));
    body.push_str("</div></div></form>");
}

const MINING_QUEUE_TRASH_ICON: &str = r#"<svg class="mining-queue-remove-icon" viewBox="0 0 24 24" width="18" height="18" focusable="false" aria-hidden="true"><path fill="currentColor" d="M9 3h6l1 1h4v2H4V4h4l1-1zm1 5h1v10h-1V8zm3 0h1v10h-1V8zM6 8h12l-1 12H7L6 8z"/></svg>"#;

fn render_queue_run_row(
    body: &mut String,
    item: &MiningQueueDisplayItem,
    show_remove_button: bool,
    refresh_on_complete: bool,
    progress_total_seconds: Option<i64>,
) {
    body.push_str(r#"<div class="mining-queue-run-row">"#);
    if show_remove_button {
        body.push_str(&format!(
            r#"<button type="button" class="mining-queue-remove-btn" data-queue-item-id="{}" aria-label="Remove queued run in {}" onclick="if(window.miningQueueRemoveRun){{window.miningQueueRemoveRun(this);}} return false;">{MINING_QUEUE_TRASH_ICON}</button>"#,
            item.mining_queue_id,
            escape_html(&item.area_name)
        ));
    }
    body.push_str(r#"<span class="mining-queue-run-area">"#);
    if active_run_result_link(item.status, item.rally_result_id) {
        body.push_str(&format!(
            r#"<a href="miningResults?rallyResultId={}">{}</a>"#,
            item.rally_result_id.unwrap_or(0),
            escape_html(&item.area_name)
        ));
    } else {
        body.push_str(&escape_html(&item.area_name));
    }
    body.push_str("</span>");
    body.push_str(&format!(
        r#"<span class="miningqueuestatus mining-queue-status mining-queue-status-{}">{}</span>"#,
        mining_queue_status_class(item.status),
        mining_queue_status_description(item.status)
    ));
    let progress_attr = progress_total_seconds
        .filter(|total| *total > 0)
        .map(|total| format!(r#" data-progress-total="{}""#, total))
        .unwrap_or_default();
    body.push_str(&format!(
        r#"<span class="miningqueuetime mining-queue-run-time" data-seconds-left="{}"{}{}>{}</span>"#,
        item.time_left_seconds,
        if refresh_on_complete {
            r#" data-refresh-on-complete="true""#
        } else {
            ""
        },
        progress_attr,
        format_queue_time_left(item.time_left_seconds)
    ));
    body.push_str("</div>");
}

fn active_run_result_link(
    status: robominer_db::MiningQueueStatus,
    rally_result_id: Option<i64>,
) -> bool {
    matches!(
        status,
        robominer_db::MiningQueueStatus::Mining | robominer_db::MiningQueueStatus::Recharging
    ) && rally_result_id.is_some()
}

fn active_run_progress_total(
    item: &MiningQueueDisplayItem,
    robot: &robominer_db::MiningQueuePageRobotRecord,
    area_map: &HashMap<i64, &robominer_db::MiningQueuePageAreaRecord>,
) -> Option<i64> {
    match item.status {
        robominer_db::MiningQueueStatus::Mining => area_map
            .get(&item.mining_area_id)
            .map(|area| i64::from(area.mining_time)),
        robominer_db::MiningQueueStatus::Recharging => Some(i64::from(robot.recharge_time)),
        _ => None,
    }
}

fn render_run_progress(body: &mut String, time_left_seconds: i64, total_seconds: i64) {
    let percent = if total_seconds > 0 {
        let elapsed = total_seconds.saturating_sub(time_left_seconds.max(0));
        ((elapsed as f64 / total_seconds as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    body.push_str(r#"<div class="mining-queue-progress" aria-hidden="true">"#);
    body.push_str(&format!(
        r#"<div class="mining-queue-progress-bar" style="width: {percent:.1}%"></div>"#
    ));
    body.push_str("</div>");
}

fn enqueue_block_reason(
    state: &MiningQueuePageState,
    queue_len: usize,
    selected_area_id: i64,
    cost_map: &HashMap<i64, Vec<&robominer_db::MiningQueuePageAreaCostRecord>>,
    ore_amount_map: &HashMap<i64, i32>,
) -> Option<String> {
    if queue_len as i64 >= i64::from(state.asset_summary.mining_queue_size) {
        return Some("Queue full for this robot.".to_string());
    }
    if !state
        .areas
        .iter()
        .any(|area| area.mining_area_id == selected_area_id)
    {
        return Some("Mining area not available.".to_string());
    }
    for cost in cost_map.get(&selected_area_id).into_iter().flatten() {
        let have = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
        if have < cost.amount {
            let need = cost.amount - have;
            return Some(format!("Need {} more {}.", need, cost.ore_name));
        }
    }
    None
}

fn mining_queue_status_class(status: robominer_db::MiningQueueStatus) -> &'static str {
    match status {
        robominer_db::MiningQueueStatus::Mining => "mining",
        robominer_db::MiningQueueStatus::Recharging => "recharging",
        robominer_db::MiningQueueStatus::Queued => "queued",
        robominer_db::MiningQueueStatus::Updating => "updating",
    }
}

pub(super) fn format_queue_time_left(seconds: i64) -> String {
    let seconds_left = seconds.max(0);
    let display_seconds = seconds_left % 60;
    let display_minutes = (seconds_left / 60) % 60;
    let display_hours = seconds_left / 3600;

    if display_hours > 0 {
        format!("{display_hours}:{display_minutes:02}:{display_seconds:02}")
    } else {
        format!("{display_minutes}:{display_seconds:02}")
    }
}

fn render_mining_queue_selection_state_inputs(
    body: &mut String,
    state: &MiningQueuePageState,
    current_robot_id: Option<i64>,
) {
    body.push_str(&format!(
        r#"<input type="hidden" name="infoMiningAreaId" value="{}"/>"#,
        state.selected_info_area_id
    ));
    for robot in &state.robots {
        if Some(robot.robot_id) == current_robot_id {
            continue;
        }
        if let Some(selected_area_id) = state.selected_robot_area_ids.get(&robot.robot_id) {
            body.push_str(&format!(
                r#"<input type="hidden" name="miningArea{}" value="{}"/>"#,
                robot.robot_id, selected_area_id
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_mining_area_details(
    body: &mut String,
    area: &robominer_db::MiningQueuePageAreaRecord,
    costs: &[&robominer_db::MiningQueuePageAreaCostRecord],
    supplies: &[&robominer_db::MiningQueuePageAreaSupplyRecord],
    yields: &[&robominer_db::MiningQueuePageAreaYieldRecord],
    robots: &[robominer_db::MiningQueuePageRobotRecord],
    score_map: &HashMap<(i64, i64), f64>,
    ore_amount_map: &HashMap<i64, i32>,
    active: bool,
) {
    let panel_class = if active {
        "mining-queue-area-panel mining-queue-area-panel-active"
    } else {
        "mining-queue-area-panel"
    };
    body.push_str(&format!(
        r#"<tbody id="miningAreaDetails{}" class="{panel_class}">"#,
        area.mining_area_id
    ));
    if !costs.is_empty() {
        body.push_str(r#"<tr><td colspan="4">Upfront costs:</td></tr>"#);
        for cost in costs {
            let user_amount = ore_amount_map.get(&cost.ore_id).copied().unwrap_or(0);
            body.push_str(&format!(
                r#"<tr><td></td><td>{}:</td><td>{}</td><td class="{}">({})</td></tr>"#,
                escape_html(&cost.ore_name),
                cost.amount,
                if user_amount >= cost.amount {
                    "sufficientbalance"
                } else {
                    "insufficientbalance"
                },
                user_amount
            ));
        }
    }
    body.push_str(&format!(
        r#"<tr><td>Tax rate:</td><td colspan="3">{}%</td></tr>"#,
        area.tax_rate
    ));
    body.push_str(&format!(
        r#"<tr><td>Mining time:</td><td colspan="3">{}</td></tr>
<tr><td>Mining cycles:</td><td colspan="3">{}</td></tr>
<tr><td>Area size:</td><td colspan="3">{} x {}</td></tr>
<tr><td colspan="4">Available ore:</td></tr>"#,
        format_period(area.mining_time),
        area.max_moves,
        area.size_x,
        area.size_y
    ));
    for supply in supplies {
        body.push_str(&format!(
            r#"<tr><td></td><td>{}:</td><td colspan="2">h {} / r {}</td></tr>"#,
            escape_html(&supply.ore_name),
            supply.supply,
            supply.radius
        ));
    }
    let mut title_added = false;
    for robot in robots {
        if let Some(score) = score_map.get(&(robot.robot_id, area.mining_area_id))
            && *score > 0.0
        {
            if !title_added {
                body.push_str(r#"<tr><td colspan="4">Robot score:</td></tr>"#);
                title_added = true;
            }
            body.push_str(&format!(
                r#"<tr><td></td><td>{}</td><td colspan="2">{:.1}</td></tr>"#,
                escape_html(&robot.robot_name),
                score
            ));
        }
    }
    body.push_str(r#"<tr><td colspan="4">Historic yield:</td></tr>"#);
    let mut total_percentage = 0.0;
    for area_yield in yields {
        total_percentage += area_yield.percentage;
        body.push_str(&format!(
            r#"<tr><td></td><td>{}:</td><td colspan="2">{:.1}%</td></tr>"#,
            escape_html(&area_yield.ore_name),
            area_yield.percentage
        ));
    }
    body.push_str(&format!(
        r#"<tr><td></td><td>Total:</td><td colspan="2">{total_percentage:.1}%</td></tr></tbody>"#
    ));
}
