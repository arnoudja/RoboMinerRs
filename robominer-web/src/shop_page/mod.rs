use crate::{
    Request, Response, ServerConfig, block_on_database, login_redirect, query_i64, session_username,
};

pub(super) const ORE_SCANNER_PART_TYPE_ID: i64 = 7;
pub(super) const MEMORY_MODULE_PART_TYPE_ID: i64 = 4;
pub(super) const ENGINE_PART_TYPE_ID: i64 = 6;

#[derive(Debug)]
pub(super) struct ShopPageState {
    pub(super) ores: Vec<robominer_db::OreRecord>,
    pub(super) part_types: Vec<robominer_db::RobotPartTypeRecord>,
    pub(super) parts: Vec<robominer_db::ShopRobotPartCatalogRecord>,
    pub(super) costs: Vec<robominer_db::ShopRobotPartCostRecord>,
    pub(super) part_states: Vec<robominer_db::ShopRobotPartStateRecord>,
    pub(super) ore_assets: Vec<robominer_db::UserOreAssetStateRecord>,
    pub(super) selected_part_type_id: i64,
    pub(super) selected_tier_id: i64,
    pub(super) selected_part_id: i64,
    pub(super) message: Option<String>,
}

pub(super) fn shop_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Shop requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let buy_part_id = query_i64(request, "buyRobotPartId");
    let sell_part_id = query_i64(request, "sellRobotPartId");
    let selected_part_type_id = query_i64(request, "selectedRobotPartTypeId");
    let selected_tier_id = query_i64(request, "selectedTierId");
    let selected_part_id = query_i64(request, "selectedRobotPartId");

    let result = block_on_database(load_shop_state(
        pool,
        user_id,
        buy_part_id,
        sell_part_id,
        request.form.contains_key("sellAllUnassigned"),
        selected_part_type_id,
        selected_tier_id,
        selected_part_id,
    ));

    match result {
        Ok(state) => Response::html(render::render_shop_page(
            session_username(request),
            crate::app_shell::hud_markup(request, config).as_deref(),
            &state,
        )),
        Err(error) => Response::service_unavailable(format!("Unable to load shop: {error}")),
    }
}

#[allow(clippy::too_many_arguments)]
async fn load_shop_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    buy_part_id: Option<i64>,
    sell_part_id: Option<i64>,
    sell_all_unassigned: bool,
    selected_part_type_id: Option<i64>,
    selected_tier_id: Option<i64>,
    selected_part_id: Option<i64>,
) -> Result<ShopPageState, robominer_domain::DomainError> {
    robominer_domain::claim_user_results(pool, user_id).await?;

    let mut message = None;
    if let Some(robot_part_id) = buy_part_id {
        message = Some(
            match robominer_domain::buy_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await?
            {
                Ok(_) => "Robot part bought".to_string(),
                Err(rejection) => format!(
                    "Unable to buy robot part: {}",
                    robot_part_transaction_rejection_message(rejection)
                ),
            },
        );
    } else if sell_all_unassigned {
        message = Some(
            match robominer_domain::sell_all_unassigned_robot_parts(pool, user_id).await? {
                Ok(result) => {
                    if result.sold_count == 1 {
                        "Sold 1 unassigned robot part".to_string()
                    } else {
                        format!("Sold {} unassigned robot parts", result.sold_count)
                    }
                }
                Err(rejection) => format!(
                    "Unable to sell robot parts: {}",
                    robot_part_transaction_rejection_message(rejection)
                ),
            },
        );
    } else if let Some(robot_part_id) = sell_part_id {
        message = Some(
            match robominer_domain::sell_robot_part(
                pool,
                robominer_db::RobotPartTransactionRequest {
                    user_id,
                    robot_part_id,
                },
            )
            .await?
            {
                Ok(_) => "Robot part sold".to_string(),
                Err(rejection) => format!(
                    "Unable to sell robot part: {}",
                    robot_part_transaction_rejection_message(rejection)
                ),
            },
        );
    }

    let ores = robominer_domain::list_shop_catalog_ores_for_user(pool, user_id).await?;
    let part_types = robominer_domain::list_shop_catalog_robot_part_types(pool).await?;
    let parts = robominer_domain::list_shop_catalog_robot_parts(pool).await?;
    let selected_part_type_id = selected_part_type_id
        .or_else(|| part_types.first().map(|part_type| part_type.id))
        .unwrap_or(0);
    let selected_tier_id = selected_tier_id
        .filter(|tier_id| ores.iter().any(|ore| ore.id == *tier_id))
        .or_else(|| default_shop_tier_id(&ores))
        .unwrap_or(0);
    let selected_part_id = resolve_selected_part_id(
        selected_part_id,
        &parts,
        selected_part_type_id,
        selected_tier_id,
    );

    Ok(ShopPageState {
        ores,
        part_types,
        parts,
        costs: robominer_domain::list_shop_catalog_robot_part_costs(pool).await?,
        part_states: robominer_domain::list_shop_robot_part_states(pool, user_id).await?,
        ore_assets: robominer_domain::list_user_ore_asset_states(pool, user_id).await?,
        selected_part_type_id,
        selected_tier_id,
        selected_part_id,
        message,
    })
}

fn default_shop_tier_id(ores: &[robominer_db::OreRecord]) -> Option<i64> {
    ores.iter().map(|ore| ore.id).max()
}

fn resolve_selected_part_id(
    selected_part_id: Option<i64>,
    parts: &[robominer_db::ShopRobotPartCatalogRecord],
    selected_part_type_id: i64,
    selected_tier_id: i64,
) -> i64 {
    if let Some(selected_part_id) = selected_part_id
        && parts
            .iter()
            .any(|part| part.robot_part_id == selected_part_id)
    {
        return selected_part_id;
    }

    parts
        .iter()
        .find(|part| part.type_id == selected_part_type_id && part.tier_id == selected_tier_id)
        .map(|part| part.robot_part_id)
        .or_else(|| parts.first().map(|part| part.robot_part_id))
        .unwrap_or(0)
}

pub(super) fn robot_part_transaction_rejection_message(
    rejection: robominer_db::RobotPartTransactionRejection,
) -> &'static str {
    robominer_domain::robot_part_transaction_rejection_message(rejection)
}

mod render;

#[cfg(test)]
mod tests;
