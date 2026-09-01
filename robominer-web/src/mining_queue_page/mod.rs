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
use view_model::{
    area_cost_view, area_supply_view, area_view, asset_summary_view,
    load_mining_queue_display_items, ore_asset_view, robot_view, score_view,
};

pub(super) const FRAGMENT_QUEUE: &str = "queue";

#[derive(Debug)]
pub(super) struct MiningQueuePageState {
    pub(super) asset_summary: MiningQueueAssetSummaryView,
    pub(super) ore_assets: Vec<MiningQueueOreAssetView>,
    pub(super) robots: Vec<MiningQueueRobotView>,
    pub(super) areas: Vec<MiningQueueAreaView>,
    pub(super) costs: Vec<MiningQueueAreaCostView>,
    pub(super) supplies: Vec<MiningQueueAreaSupplyView>,
    pub(super) scores: Vec<MiningQueueScoreView>,
    pub(super) items: Vec<MiningQueueDisplayItem>,
    pub(super) selected_info_area_id: i64,
    pub(super) selected_robot_area_ids: HashMap<i64, i64>,
    pub(super) error_message: Option<String>,
}

#[derive(Debug)]
pub(super) struct MiningQueueAssetSummaryView {
    pub(super) mining_queue_size: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueOreAssetView {
    pub(super) ore_id: i64,
    pub(super) ore_name: String,
    pub(super) amount: i32,
    pub(super) max_allowed: i32,
    pub(super) depot_max_allowed: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueRobotView {
    pub(super) robot_id: i64,
    pub(super) robot_name: String,
    pub(super) recharge_time: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueAreaView {
    pub(super) mining_area_id: i64,
    pub(super) area_name: String,
    pub(super) tax_rate: i32,
    pub(super) depot_tax_rate: i32,
    pub(super) mining_time: i32,
    pub(super) max_moves: i32,
    pub(super) size_x: i32,
    pub(super) size_y: i32,
    pub(super) score_ore_target: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueAreaCostView {
    pub(super) mining_area_id: i64,
    pub(super) ore_id: i64,
    pub(super) ore_name: String,
    pub(super) amount: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueAreaSupplyView {
    pub(super) mining_area_id: i64,
    pub(super) ore_id: i64,
    pub(super) ore_name: String,
    pub(super) supply: i32,
    pub(super) radius: i32,
}

#[derive(Debug)]
pub(super) struct MiningQueueScoreView {
    pub(super) robot_id: i64,
    pub(super) mining_area_id: i64,
    pub(super) score: f64,
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

pub(super) async fn mining_queue_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
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
                    if let robominer_domain::EnqueueMiningOutcome::Rejected(rejection) =
                        robominer_domain::enqueue_mining(
                            pool,
                            robominer_db::EnqueueMiningRequest {
                                user_id,
                                robot_id,
                                mining_area_id,
                                fill,
                            },
                        )
                        .await
                        .map_err(|error| {
                            crate::page_context::PageLoadError::from_database(error).unwrap_or_else(
                                |_| {
                                    crate::page_context::PageLoadError::from(
                                        sqlx::Error::Configuration(
                                            "unexpected domain error on enqueue mining".into(),
                                        ),
                                    )
                                },
                            )
                        })?
                    {
                        error_message =
                            Some(robominer_domain::rejection_messages::enqueue_mining_rejection_player_message(rejection).to_string());
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

    let asset_summary =
        asset_summary_view(robominer_db::load_user_asset_summary(pool, user_id).await?);
    let robots = robominer_db::list_mining_queue_page_robots(pool, user_id)
        .await?
        .into_iter()
        .map(robot_view)
        .collect::<Vec<_>>();
    let areas = robominer_db::list_mining_queue_page_areas(pool, user_id)
        .await?
        .into_iter()
        .map(area_view)
        .collect::<Vec<_>>();
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
        ore_assets: robominer_db::list_user_ore_asset_states(pool, user_id)
            .await?
            .into_iter()
            .map(ore_asset_view)
            .collect(),
        robots,
        areas,
        costs: robominer_db::list_mining_queue_page_area_costs(pool, user_id)
            .await?
            .into_iter()
            .map(area_cost_view)
            .collect(),
        supplies: robominer_db::list_mining_queue_page_area_supplies(pool, user_id)
            .await?
            .into_iter()
            .map(area_supply_view)
            .collect(),
        scores: robominer_db::list_robot_mining_area_scores_for_user(pool, user_id)
            .await?
            .into_iter()
            .map(score_view)
            .collect(),
        items,
        selected_info_area_id,
        selected_robot_area_ids,
        error_message,
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
