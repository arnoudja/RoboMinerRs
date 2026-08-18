use crate::{Request, Response, ServerConfig, query_i64, rally_pages};

const MINING_RESULTS_MAX_SHOWN: i64 = 50;
pub(super) const MINING_RESULTS_INITIAL_VISIBLE: usize = 5;
pub(super) const MINING_RESULTS_LOAD_MORE_STEP: usize = 5;

#[derive(Debug)]
pub(super) struct MiningResultsPageState {
    pub(super) robots: Vec<robominer_db::MiningQueuePageRobotRecord>,
    pub(super) results: Vec<robominer_db::MiningResultStateRecord>,
    pub(super) ore_results: Vec<robominer_db::MiningResultOreStateRecord>,
    pub(super) action_results: Vec<robominer_db::MiningResultActionStateRecord>,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
    pub(super) selected_mining_queue_id: Option<i64>,
}

pub(super) async fn mining_results_page(request: &Request, config: &ServerConfig) -> Response {
    let session = match crate::page_context::PageSession::require_read(
        request,
        config,
        "Mining results require ROBOMINER_DATABASE_URL to be configured",
    ) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if let Some(rally_result_id) = query_i64(request, "rallyResultId") {
        let result =
            rally_pages::load_user_rally_view_state(session.pool, session.user_id, rally_result_id)
                .await;

        return match result {
            Ok(Some(state)) => {
                session
                    .html_read_with_hud(request, config, |username, hud| {
                        rally_pages::render_rally_view_page(
                            username,
                            hud,
                            &state,
                            request
                                .query
                                .get("returnTo")
                                .map(String::as_str)
                                .and_then(rally_pages::valid_mining_results_return_to)
                                .map(rally_pages::RallyViewBackLink::MiningResults),
                        )
                    })
                    .await
            }
            Ok(None) => Response::not_found(),
            Err(error) => crate::page_context::page_load_error("rally view", error),
        };
    }

    let preferred_run_id = query_i64(request, "runId");
    let result = load_mining_results_state(
        session.pool,
        session.user_id,
        MINING_RESULTS_MAX_SHOWN,
        preferred_run_id,
    )
    .await;

    match result {
        Ok(state) => {
            session
                .html_read_with_hud(request, config, |username, hud| {
                    render::render_mining_results_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("mining results", error),
    }
}

async fn load_mining_results_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    max_results: i64,
    preferred_run_id: Option<i64>,
) -> Result<MiningResultsPageState, crate::page_context::PageLoadError> {
    let claim_result = crate::page_context::claim_user_results(pool, user_id).await?;

    let results =
        robominer_db::list_mining_result_states_for_user(pool, user_id, max_results).await?;

    Ok(MiningResultsPageState {
        robots: robominer_db::list_mining_queue_page_robots(pool, user_id).await?,
        selected_mining_queue_id: selected_mining_queue_id(&results, preferred_run_id),
        results,
        ore_results: robominer_db::list_mining_result_ore_states_for_user(
            pool,
            user_id,
            max_results,
        )
        .await?,
        action_results: robominer_db::list_mining_result_action_states_for_user(
            pool,
            user_id,
            max_results,
        )
        .await?,
        claimed_results: claim_result,
    })
}

pub(super) fn selected_mining_queue_id(
    results: &[robominer_db::MiningResultStateRecord],
    preferred_run_id: Option<i64>,
) -> Option<i64> {
    if let Some(run_id) = preferred_run_id
        && results
            .iter()
            .any(|result| result.mining_queue_id == run_id)
    {
        return Some(run_id);
    }
    results.first().map(|result| result.mining_queue_id)
}

mod render;
mod render_detail;
mod render_filters;
mod render_log;
mod scripts;

#[cfg(test)]
mod tests;
