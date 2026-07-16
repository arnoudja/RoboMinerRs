use std::collections::HashMap;

use crate::help_pages;
use crate::html::{escape_html, layout, selected_attr};
use crate::mining_queue_page::{MiningQueueDisplayItem, MiningQueuePageState};

use super::inspector::render_mining_area_details;
use super::robots::{render_robot_card, render_wallet_strip};

#[allow(unused_imports)]
pub(super) use super::robots::format_queue_time_left;

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
    body.push_str(super::scripts::MINING_QUEUE_PAGE_SCRIPT);
    body.push_str("</div>");

    layout(
        "RoboMiner - Mining queue",
        "miningQueue",
        &username,
        hud,
        &body,
    )
}
