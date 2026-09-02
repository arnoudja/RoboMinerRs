use std::collections::HashMap;

use crate::html::{EscapedHtml, html_attr, optional_title_attr, selected_attr};
use crate::mining_queue_page::{
    MiningQueueAreaCostView, MiningQueueAreaView, MiningQueueDisplayItem, MiningQueuePageState,
    MiningQueueRobotView,
};

use super::enqueue_block_reason;
use super::queue_row::{active_run_progress_total, render_queue_run_row, render_run_progress};
use crate::mining_queue_page::inspector::render_mining_queue_selection_state_inputs;

pub(in crate::mining_queue_page) fn render_robot_card(
    body: &mut String,
    state: &MiningQueuePageState,
    robot: &MiningQueueRobotView,
    queue_items: &[&MiningQueueDisplayItem],
    cost_map: &HashMap<i64, Vec<&MiningQueueAreaCostView>>,
    ore_amount_map: &HashMap<i64, i32>,
    area_map: &HashMap<i64, &MiningQueueAreaView>,
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
    let title_attr = optional_title_attr(selected_enqueue_block_reason.as_deref());

    body.push_str(&format!(
        r#"<form action="miningQueue" method="post" class="mining-queue-card" data-robot-id="{}"><input type="hidden" name="robotId" value="{}"/>"#,
        robot.robot_id, robot.robot_id
    ));
    render_mining_queue_selection_state_inputs(body, state, Some(robot.robot_id));

    body.push_str(r#"<div class="mining-queue-card-status">"#);
    body.push_str(r#"<div class="mining-queue-card-header">"#);
    body.push_str(&format!(
        r#"<h2 class="mining-queue-robot-name"><a href="robot?robotId={}">{}</a></h2>"#,
        robot.robot_id,
        EscapedHtml::from(robot.robot_name.as_str())
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
            render_run_progress(
                body,
                active_item.time_left_seconds.saturating_add(1),
                total_seconds.saturating_add(1),
            );
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
    body.push_str("</div>");

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
            .map(|reason| format!(r#" data-block-reason="{}""#, html_attr(reason)))
            .unwrap_or_default();
        body.push_str(&format!(
            r#"<option value="{}"{}{}>{}</option>"#,
            area.mining_area_id,
            selected_attr(area.mining_area_id == selected_area_id),
            block_reason_attr,
            EscapedHtml::from(area.area_name.as_str())
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
    let clearable_count = queue_items
        .iter()
        .filter(|item| item.status == robominer_db::MiningQueueStatus::Queued)
        .count();
    let clear_disabled = if clearable_count == 0 {
        " disabled"
    } else {
        ""
    };
    let clear_title = if clearable_count == 0 {
        r#" title="No queued runs to clear""#
    } else {
        ""
    };
    body.push_str(&format!(
        r#"<button type="button" class="mining-queue-btn mining-queue-clear-btn" data-clearable-count="{clearable_count}"{clear_disabled}{clear_title}>Clear queue</button>"#
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
        EscapedHtml::from(selected_enqueue_block_reason.as_deref().unwrap_or(""))
    ));
    body.push_str("</div></div></form>");
}
