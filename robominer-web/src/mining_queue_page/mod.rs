use std::collections::HashMap;

use crate::{Request, Response, ServerConfig, is_post, mutation_i64, query_i64};

mod actions;
mod inspector;
mod render;
mod robots;
mod scripts;
mod view_model;

#[cfg(test)]
mod tests;

use actions::{cancel_queued_items, format_cancel_batch_message};
use view_model::load_mining_queue_display_items;

pub(super) const FRAGMENT_QUEUE: &str = "queue";

#[derive(Debug)]
pub(super) struct MiningQueuePageState {
    pub(super) asset_summary: robominer_db::UserAssetSummaryRecord,
    pub(super) ore_assets: Vec<robominer_db::UserOreAssetStateRecord>,
    pub(super) robots: Vec<robominer_db::MiningQueuePageRobotRecord>,
    pub(super) areas: Vec<robominer_db::MiningQueuePageAreaRecord>,
    pub(super) costs: Vec<robominer_db::MiningQueuePageAreaCostRecord>,
    pub(super) supplies: Vec<robominer_db::MiningQueuePageAreaSupplyRecord>,
    pub(super) scores: Vec<robominer_db::RobotMiningAreaScoreRecord>,
    pub(super) items: Vec<MiningQueueDisplayItem>,
    pub(super) selected_info_area_id: i64,
    pub(super) selected_robot_area_ids: HashMap<i64, i64>,
    pub(super) error_message: Option<String>,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
}

#[derive(Debug)]
pub(super) struct MiningQueueDisplayItem {
    pub(super) mining_queue_id: i64,
    pub(super) robot_id: i64,
    pub(super) mining_area_id: i64,
    pub(super) area_name: String,
    pub(super) rally_result_id: Option<i64>,
    pub(super) status: robominer_db::MiningQueueStatus,
    pub(super) time_left_seconds: i64,
}

pub(super) async fn mining_queue_page(request: &Request, config: &ServerConfig) -> Response {
    let session = match crate::page_context::PageSession::require(
        request,
        config,
        "Mining queue requires ROBOMINER_DATABASE_URL to be configured",
    ) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let selected_queue_item_ids = if is_post(request) {
        form_i64_values(request, "selectedQueueItemId")
    } else {
        Vec::new()
    };
    let result = load_mining_queue_page_state(
        session.pool,
        session.user_id,
        request,
        selected_queue_item_ids,
    )
    .await;

    match result {
        Ok(state) => {
            if wants_queue_fragment(request) {
                let hud = crate::app_shell::hud_markup(request, config)
                    .await
                    .unwrap_or_default();
                let html = render::render_mining_queue_fragment(&hud, &state);
                return crate::csrf::html_with_csrf(request, session.user_id, html);
            }
            session
                .html_with_hud(request, config, |username, hud| {
                    render::render_mining_queue_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("mining queue", error),
    }
}

async fn load_mining_queue_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
    selected_queue_item_ids: Vec<i64>,
) -> Result<MiningQueuePageState, crate::page_context::PageLoadError> {
    let claim_result = crate::page_context::claim_user_results(pool, user_id).await?;

    let mut error_message = None;
    if is_post(request) {
        match request.form.get("submitType").map(String::as_str) {
            Some("add") | Some("fill") => {
                let robot_id = mutation_i64(request, "robotId").unwrap_or(0);
                let mining_area_id =
                    mutation_i64(request, &format!("miningArea{}", robot_id)).unwrap_or(0);
                if robot_id <= 0 {
                    error_message = Some("Unknown robot".to_string());
                } else if mining_area_id <= 0 {
                    error_message = Some("Unknown mining area".to_string());
                } else {
                    let fill = request
                        .form
                        .get("submitType")
                        .is_some_and(|value| value == "fill");
                    if let Err(rejection) = robominer_db::enqueue_mining(
                        pool,
                        robominer_db::EnqueueMiningRequest {
                            user_id,
                            robot_id,
                            mining_area_id,
                            fill,
                        },
                    )
                    .await?
                    .into_result()
                    {
                        error_message =
                            Some(enqueue_mining_rejection_message(rejection).to_string());
                    }
                }
            }
            Some("remove") => {
                let robot_id = mutation_i64(request, "robotId").unwrap_or(0);
                if robot_id > 0 {
                    let items = load_mining_queue_display_items(pool, user_id).await?;
                    let queue_ids: Vec<i64> = items
                        .iter()
                        .filter(|item| {
                            item.robot_id == robot_id
                                && item.status == robominer_db::MiningQueueStatus::Queued
                                && selected_queue_item_ids.contains(&item.mining_queue_id)
                        })
                        .map(|item| item.mining_queue_id)
                        .collect();
                    let batch = cancel_queued_items(pool, user_id, &queue_ids, false).await?;
                    error_message = format_cancel_batch_message(&batch);
                }
            }
            Some("clear") => {
                let robot_id = mutation_i64(request, "robotId").unwrap_or(0);
                if robot_id > 0 {
                    let require_refund_fits = request
                        .form
                        .get("clearMode")
                        .is_some_and(|value| value == "safe");
                    let items = load_mining_queue_display_items(pool, user_id).await?;
                    let queue_ids: Vec<i64> = items
                        .iter()
                        .filter(|item| {
                            item.robot_id == robot_id
                                && item.status == robominer_db::MiningQueueStatus::Queued
                                && (selected_queue_item_ids.is_empty()
                                    || selected_queue_item_ids.contains(&item.mining_queue_id))
                        })
                        .map(|item| item.mining_queue_id)
                        .collect();
                    let batch =
                        cancel_queued_items(pool, user_id, &queue_ids, require_refund_fits).await?;
                    error_message = format_cancel_batch_message(&batch);
                }
            }
            _ => {}
        }
    }

    let asset_summary = robominer_db::load_user_asset_summary(pool, user_id).await?;
    let robots = robominer_db::list_mining_queue_page_robots(pool, user_id).await?;
    let areas = robominer_db::list_mining_queue_page_areas(pool, user_id).await?;
    let items = load_mining_queue_display_items(pool, user_id).await?;
    let fallback_area_id = areas.first().map(|area| area.mining_area_id).unwrap_or(0);
    let selected_info_area_id = query_i64(request, "infoMiningAreaId").unwrap_or(fallback_area_id);
    let mut selected_robot_area_ids = HashMap::new();
    for robot in &robots {
        let selected_area_id = query_i64(request, &format!("miningArea{}", robot.robot_id))
            .unwrap_or(fallback_area_id);
        selected_robot_area_ids.insert(robot.robot_id, selected_area_id);
    }

    Ok(MiningQueuePageState {
        asset_summary,
        ore_assets: robominer_db::list_user_ore_asset_states(pool, user_id).await?,
        robots,
        areas,
        costs: robominer_db::list_mining_queue_page_area_costs(pool, user_id).await?,
        supplies: robominer_db::list_mining_queue_page_area_supplies(pool, user_id).await?,
        scores: robominer_db::list_robot_mining_area_scores_for_user(pool, user_id).await?,
        items,
        selected_info_area_id,
        selected_robot_area_ids,
        error_message,
        claimed_results: claim_result,
    })
}

fn form_i64_values(request: &Request, name: &str) -> Vec<i64> {
    request
        .form_values
        .get(name)
        .into_iter()
        .flatten()
        .filter_map(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .collect()
}

fn wants_queue_fragment(request: &Request) -> bool {
    request
        .query
        .get("fragment")
        .or_else(|| request.form.get("fragment"))
        .is_some_and(|value| value == FRAGMENT_QUEUE)
}

pub(super) fn mining_queue_status_description(
    status: robominer_db::MiningQueueStatus,
) -> &'static str {
    match status {
        robominer_db::MiningQueueStatus::Mining => "Mining",
        robominer_db::MiningQueueStatus::Recharging => "Recharging",
        robominer_db::MiningQueueStatus::Queued => "Waiting for rally",
        robominer_db::MiningQueueStatus::Updating => "Finishing rally",
    }
}

pub(super) fn enqueue_mining_rejection_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    robominer_domain::rejection_messages::enqueue_mining_rejection_player_message(rejection)
}

pub(super) fn cancel_mining_rejection_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    robominer_domain::rejection_messages::cancel_mining_queue_rejection_player_message(rejection)
}
