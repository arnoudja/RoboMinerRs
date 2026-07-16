use std::collections::HashMap;

use crate::html::{escape_html, selected_attr};
use crate::mining_queue_page::{MiningQueueDisplayItem, MiningQueuePageState};

use super::inspector::render_mining_queue_selection_state_inputs;
use super::mining_queue_status_description;

pub(super) fn render_wallet_strip(body: &mut String, state: &MiningQueuePageState) {
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

pub(super) fn render_robot_card(
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

pub(super) fn render_queue_run_row(
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

pub(super) fn active_run_result_link(
    status: robominer_db::MiningQueueStatus,
    rally_result_id: Option<i64>,
) -> bool {
    matches!(
        status,
        robominer_db::MiningQueueStatus::Mining | robominer_db::MiningQueueStatus::Recharging
    ) && rally_result_id.is_some()
}

pub(super) fn active_run_progress_total(
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

pub(super) fn render_run_progress(body: &mut String, time_left_seconds: i64, total_seconds: i64) {
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

pub(super) fn enqueue_block_reason(
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

pub(super) fn mining_queue_status_class(status: robominer_db::MiningQueueStatus) -> &'static str {
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
