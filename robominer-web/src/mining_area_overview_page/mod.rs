use crate::{Request, Response, ServerConfig, session_username};

mod render;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct MiningAreaOverviewPageState {
    pub(super) ores: Vec<robominer_db::MiningAreaOverviewOreRecord>,
    pub(super) areas: Vec<robominer_db::MiningAreaOverviewAreaRecord>,
    pub(super) ore_averages: Vec<robominer_db::MiningAreaOverviewOreAverageRecord>,
    pub(super) costs: Vec<robominer_db::MiningQueuePageAreaCostRecord>,
    pub(super) ore_assets: Vec<robominer_db::UserOreAssetStateRecord>,
}

pub(super) async fn mining_area_overview_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    let result = load_mining_area_overview_state(session.pool, session.user_id).await;

    match result {
        Ok(state) => Response::html(render::render_mining_area_overview_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config)
                .await
                .as_deref(),
            &state,
        )),
        Err(error) => crate::page_context::page_load_error("mining area overview", error),
    }
}

async fn load_mining_area_overview_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<MiningAreaOverviewPageState, crate::page_context::PageLoadError> {
    Ok(MiningAreaOverviewPageState {
        ores: robominer_db::list_mining_area_overview_ores_for_user(pool, user_id).await?,
        areas: robominer_db::list_mining_area_overview_areas_for_user(pool, user_id).await?,
        ore_averages: robominer_db::list_mining_area_overview_ore_averages_for_user(pool, user_id)
            .await?,
        costs: robominer_db::list_mining_queue_page_area_costs(pool, user_id).await?,
        ore_assets: robominer_db::list_user_ore_asset_states(pool, user_id).await?,
    })
}
