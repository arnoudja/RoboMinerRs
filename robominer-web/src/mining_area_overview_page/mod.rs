use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, session_username,
};

mod render;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct MiningAreaOverviewPageState {
    pub(super) ores: Vec<robominer_db::MiningAreaOverviewOreRecord>,
    pub(super) areas: Vec<robominer_db::MiningAreaOverviewAreaRecord>,
    pub(super) percentages: Vec<robominer_db::MiningAreaOverviewPercentageRecord>,
    pub(super) costs: Vec<robominer_db::MiningQueuePageAreaCostRecord>,
    pub(super) ore_assets: Vec<robominer_db::UserOreAssetStateRecord>,
}

pub(super) fn mining_area_overview_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Mining area overview requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let result = block_on_database(load_mining_area_overview_state(pool, user_id));

    match result {
        Ok(state) => Response::html(render::render_mining_area_overview_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => {
            Response::service_unavailable(format!("Unable to load mining area overview: {error}"))
        }
    }
}

async fn load_mining_area_overview_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<MiningAreaOverviewPageState, robominer_domain::DomainError> {
    Ok(MiningAreaOverviewPageState {
        ores: robominer_domain::list_mining_area_overview_ores_for_user(pool, user_id).await?,
        areas: robominer_domain::list_mining_area_overview_areas_for_user(pool, user_id).await?,
        percentages: robominer_domain::list_mining_area_overview_percentages_for_user(pool, user_id)
            .await?,
        costs: robominer_domain::list_mining_queue_page_area_costs(pool, user_id).await?,
        ore_assets: robominer_domain::list_user_ore_asset_states(pool, user_id).await?,
    })
}
