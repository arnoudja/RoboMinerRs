use robominer_db::MySqlPool;

use crate::error::DomainError;

pub async fn buy_robot_part(
    pool: &MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<
    Result<robominer_db::RobotPartTransaction, robominer_db::RobotPartTransactionRejection>,
    DomainError,
> {
    robominer_db::buy_robot_part(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn sell_robot_part(
    pool: &MySqlPool,
    request: robominer_db::RobotPartTransactionRequest,
) -> Result<
    Result<robominer_db::RobotPartTransaction, robominer_db::RobotPartTransactionRejection>,
    DomainError,
> {
    robominer_db::sell_robot_part(pool, request)
        .await
        .map_err(DomainError::Database)
}

pub async fn sell_all_unassigned_robot_parts(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<
    Result<
        robominer_db::SellAllUnassignedRobotPartsResult,
        robominer_db::RobotPartTransactionRejection,
    >,
    DomainError,
> {
    robominer_db::sell_all_unassigned_robot_parts(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_robot_part_states(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::ShopRobotPartStateRecord>, DomainError> {
    robominer_db::list_shop_robot_part_states(pool, user_id)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_catalog_ores(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::OreRecord>, DomainError> {
    robominer_db::list_ores(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_catalog_ores_for_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<robominer_db::OreRecord>, DomainError> {
    robominer_db::list_mining_area_overview_ores_for_user(pool, user_id)
        .await
        .map_err(DomainError::Database)
        .map(shop_catalog_ore_records)
}

pub(crate) fn shop_catalog_ore_records(
    ores: Vec<robominer_db::MiningAreaOverviewOreRecord>,
) -> Vec<robominer_db::OreRecord> {
    ores.into_iter()
        .map(|ore| robominer_db::OreRecord {
            id: ore.ore_id,
            ore_name: ore.ore_name,
        })
        .collect()
}

pub async fn list_ores(pool: &MySqlPool) -> Result<Vec<robominer_db::OreRecord>, DomainError> {
    robominer_db::list_ores(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_catalog_robot_part_types(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::RobotPartTypeRecord>, DomainError> {
    robominer_db::list_robot_part_types(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_catalog_robot_parts(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::ShopRobotPartCatalogRecord>, DomainError> {
    robominer_db::list_shop_robot_part_catalog(pool)
        .await
        .map_err(DomainError::Database)
}

pub async fn list_shop_catalog_robot_part_costs(
    pool: &MySqlPool,
) -> Result<Vec<robominer_db::ShopRobotPartCostRecord>, DomainError> {
    robominer_db::list_shop_robot_part_costs(pool)
        .await
        .map_err(DomainError::Database)
}

#[cfg(test)]
mod tests {
    use super::shop_catalog_ore_records;

    #[test]
    fn shop_catalog_ore_records_maps_overview_rows() {
        let records = shop_catalog_ore_records(vec![
            robominer_db::MiningAreaOverviewOreRecord {
                ore_id: 7,
                ore_name: "Cerbonium".to_string(),
            },
        ]);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 7);
        assert_eq!(records[0].ore_name, "Cerbonium");
    }
}
