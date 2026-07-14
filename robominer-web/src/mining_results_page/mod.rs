use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, query_i64, rally_pages,
    session_username,
};

const MINING_RESULTS_MAX_SHOWN: i64 = 10;

#[derive(Debug)]
pub(super) struct MiningResultsPageState {
    pub(super) robots: Vec<robominer_db::MiningQueuePageRobotRecord>,
    pub(super) results: Vec<robominer_db::MiningResultStateRecord>,
    pub(super) ore_results: Vec<robominer_db::MiningResultOreStateRecord>,
    pub(super) action_results: Vec<robominer_db::MiningResultActionStateRecord>,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
    pub(super) selected_mining_queue_id: Option<i64>,
}

pub(super) fn mining_results_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Mining results require ROBOMINER_DATABASE_URL to be configured",
        );
    };

    if let Some(rally_result_id) = query_i64(request, "rallyResultId") {
        let result = block_on_database(rally_pages::load_user_rally_view_state(
            pool,
            user_id,
            rally_result_id,
        ));

        return match result {
            Ok(Some(state)) => Response::html(rally_pages::render_rally_view_page(
                session_username(request),
                crate::app_shell::hud_markup(request, config).as_deref(),
                &state,
                request
                    .query
                    .get("returnTo")
                    .map(String::as_str)
                    .and_then(rally_pages::valid_mining_results_return_to)
                    .map(rally_pages::RallyViewBackLink::MiningResults),
            )),
            Ok(None) => Response::not_found(),
            Err(error) => {
                Response::service_unavailable(format!("Unable to load rally view: {error}"))
            }
        };
    }

    let preferred_run_id = query_i64(request, "runId");
    let result = block_on_database(load_mining_results_state(
        pool,
        user_id,
        MINING_RESULTS_MAX_SHOWN,
        preferred_run_id,
    ));

    match result {
        Ok(state) => Response::html(render::render_mining_results_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => {
            Response::service_unavailable(format!("Unable to load mining results: {error}"))
        }
    }
}

async fn load_mining_results_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    max_results: i64,
    preferred_run_id: Option<i64>,
) -> Result<MiningResultsPageState, robominer_domain::DomainError> {
    let claim_result = robominer_domain::claim_user_results(pool, user_id).await?;

    let results = robominer_domain::list_mining_result_states(pool, user_id, max_results).await?;

    Ok(MiningResultsPageState {
        robots: robominer_domain::list_mining_queue_page_robots(pool, user_id).await?,
        selected_mining_queue_id: selected_mining_queue_id(&results, preferred_run_id),
        results,
        ore_results: robominer_domain::list_mining_result_ore_states(pool, user_id, max_results)
            .await?,
        action_results: robominer_domain::list_mining_result_action_states(
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

#[cfg(test)]
mod tests;
