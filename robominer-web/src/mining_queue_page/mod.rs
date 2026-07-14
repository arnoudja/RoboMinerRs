use std::collections::HashMap;

use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, query_i64, session_username,
};

#[derive(Debug)]
pub(super) struct MiningQueuePageState {
    pub(super) asset_summary: robominer_db::UserAssetSummaryRecord,
    pub(super) ore_assets: Vec<robominer_db::UserOreAssetStateRecord>,
    pub(super) robots: Vec<robominer_db::MiningQueuePageRobotRecord>,
    pub(super) areas: Vec<robominer_db::MiningQueuePageAreaRecord>,
    pub(super) costs: Vec<robominer_db::MiningQueuePageAreaCostRecord>,
    pub(super) supplies: Vec<robominer_db::MiningQueuePageAreaSupplyRecord>,
    pub(super) yields: Vec<robominer_db::MiningQueuePageAreaYieldRecord>,
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

pub(super) fn mining_queue_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Mining queue requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let selected_queue_item_ids = form_i64_values(request, "selectedQueueItemId");
    let result = block_on_database(load_mining_queue_page_state(
        pool,
        user_id,
        request,
        selected_queue_item_ids,
    ));

    match result {
        Ok(state) => Response::html(render::render_mining_queue_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => {
            Response::service_unavailable(format!("Unable to load mining queue: {error}"))
        }
    }
}

async fn load_mining_queue_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
    selected_queue_item_ids: Vec<i64>,
) -> Result<MiningQueuePageState, robominer_domain::DomainError> {
    let claim_result = robominer_domain::claim_user_results(pool, user_id).await?;

    let mut error_message = None;
    match request.form.get("submitType").map(String::as_str) {
        Some("add") | Some("fill") => {
            let robot_id = query_i64(request, "robotId").unwrap_or(0);
            let mining_area_id =
                query_i64(request, &format!("miningArea{}", robot_id)).unwrap_or(0);
            if robot_id <= 0 {
                error_message = Some("Unknown robot".to_string());
            } else if mining_area_id <= 0 {
                error_message = Some("Unknown mining area".to_string());
            } else {
                let fill = request
                    .form
                    .get("submitType")
                    .is_some_and(|value| value == "fill");
                if let Err(rejection) = robominer_domain::enqueue_mining(
                    pool,
                    robominer_db::EnqueueMiningRequest {
                        user_id,
                        robot_id,
                        mining_area_id,
                        fill,
                    },
                )
                .await?
                {
                    error_message = Some(enqueue_mining_rejection_message(rejection).to_string());
                }
            }
        }
        Some("remove") => {
            let robot_id = query_i64(request, "robotId").unwrap_or(0);
            if robot_id > 0 {
                let items = load_mining_queue_display_items(pool, user_id).await?;
                for item in items.iter().filter(|item| {
                    item.robot_id == robot_id
                        && item.status == robominer_db::MiningQueueStatus::Queued
                        && selected_queue_item_ids.contains(&item.mining_queue_id)
                }) {
                    match robominer_domain::cancel_mining_queue(
                        pool,
                        robominer_db::CancelMiningQueueRequest {
                            user_id,
                            mining_queue_id: item.mining_queue_id,
                        },
                    )
                    .await?
                    {
                        Ok(_) => {}
                        Err(rejection) => {
                            error_message =
                                Some(cancel_mining_rejection_message(rejection).to_string());
                        }
                    }
                }
            }
        }
        _ => {}
    }

    let asset_summary = robominer_domain::load_user_asset_summary(pool, user_id).await?;
    let robots = robominer_domain::list_mining_queue_page_robots(pool, user_id).await?;
    let areas = robominer_domain::list_mining_queue_page_areas(pool, user_id).await?;
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
        ore_assets: robominer_domain::list_user_ore_asset_states(pool, user_id).await?,
        robots,
        areas,
        costs: robominer_domain::list_mining_queue_page_area_costs(pool, user_id).await?,
        supplies: robominer_domain::list_mining_queue_page_area_supplies(pool, user_id).await?,
        yields: robominer_domain::list_mining_queue_page_area_yields(pool, user_id).await?,
        scores: robominer_domain::list_robot_mining_area_scores(pool, user_id).await?,
        items,
        selected_info_area_id,
        selected_robot_area_ids,
        error_message,
        claimed_results: claim_result,
    })
}

async fn load_mining_queue_display_items(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<Vec<MiningQueueDisplayItem>, robominer_domain::DomainError> {
    let items = robominer_domain::list_mining_queue_page_items(pool, user_id).await?;
    let states = robominer_domain::list_mining_queue_states(pool, user_id).await?;
    let state_map: HashMap<i64, robominer_db::MiningQueueStateRecord> = states
        .into_iter()
        .map(|state| (state.mining_queue_id, state))
        .collect();

    Ok(items
        .into_iter()
        .map(|item| {
            let state = state_map.get(&item.mining_queue_id);
            MiningQueueDisplayItem {
                mining_queue_id: item.mining_queue_id,
                robot_id: item.robot_id,
                mining_area_id: item.mining_area_id,
                area_name: item.area_name,
                rally_result_id: item.rally_result_id,
                status: state
                    .map(|state| state.status)
                    .unwrap_or(robominer_db::MiningQueueStatus::Queued),
                time_left_seconds: state
                    .map(|state| state.time_left_seconds)
                    .unwrap_or_default(),
            }
        })
        .collect())
}

mod render;

#[cfg(test)]
mod tests;
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

pub(super) fn mining_queue_status_description(
    status: robominer_db::MiningQueueStatus,
) -> &'static str {
    match status {
        robominer_db::MiningQueueStatus::Mining => "Mining",
        robominer_db::MiningQueueStatus::Recharging => "Recharging",
        robominer_db::MiningQueueStatus::Queued => "Waiting for rally",
        robominer_db::MiningQueueStatus::Updating => "Applying robot changes",
    }
}

pub(super) fn enqueue_mining_rejection_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    robominer_domain::enqueue_mining_rejection_player_message(rejection)
}

pub(super) fn cancel_mining_rejection_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    robominer_domain::cancel_mining_queue_rejection_player_message(rejection)
}
