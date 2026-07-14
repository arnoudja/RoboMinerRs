use sqlx::MySqlPool;

use crate::mappers::{robot_part_record, shop_robot_part_catalog_record};
use crate::{
    OreRecord, RobotPartRecord, RobotPartTypeRecord, ShopRobotPartCatalogRecord,
    ShopRobotPartCostRecord,
};

pub async fn list_robot_part_types(
    pool: &MySqlPool,
) -> Result<Vec<RobotPartTypeRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, typeName \
         FROM RobotPartType \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(id, type_name)| RobotPartTypeRecord { id, type_name })
            .collect()
    })
}

pub async fn list_ores(pool: &MySqlPool) -> Result<Vec<OreRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, oreName \
         FROM Ore \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(id, ore_name)| OreRecord { id, ore_name })
            .collect()
    })
}

pub async fn list_shop_robot_part_catalog(
    pool: &MySqlPool,
) -> Result<Vec<ShopRobotPartCatalogRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT RobotPart.id, RobotPart.typeId, RobotPart.tierId, Ore.oreName, \
                RobotPart.partName, RobotPart.oreCapacity, RobotPart.miningCapacity, \
                RobotPart.batteryCapacity, RobotPart.memoryCapacity, RobotPart.cpuCapacity, \
                RobotPart.forwardCapacity, RobotPart.backwardCapacity, \
                RobotPart.rotateCapacity, RobotPart.rechargeTime, RobotPart.scanTime, \
                RobotPart.scanDistance, RobotPart.weight, RobotPart.volume, RobotPart.powerUsage \
         FROM RobotPart \
         INNER JOIN Ore ON Ore.id = RobotPart.tierId \
         ORDER BY RobotPart.typeId, RobotPart.id",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(shop_robot_part_catalog_record)
        .collect()
}

pub async fn list_shop_robot_part_costs(
    pool: &MySqlPool,
) -> Result<Vec<ShopRobotPartCostRecord>, sqlx::Error> {
    sqlx::query_as::<_, (i64, i64, String, i32)>(
        "SELECT RobotPart.id, OrePriceAmount.oreId, Ore.oreName, OrePriceAmount.amount \
         FROM RobotPart \
         INNER JOIN OrePriceAmount ON OrePriceAmount.orePriceId = RobotPart.orePriceId \
         INNER JOIN Ore ON Ore.id = OrePriceAmount.oreId \
         ORDER BY RobotPart.id, OrePriceAmount.oreId",
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(robot_part_id, ore_id, ore_name, amount)| ShopRobotPartCostRecord {
                    robot_part_id,
                    ore_id,
                    ore_name,
                    amount,
                },
            )
            .collect()
    })
}

pub async fn get_robot_part(
    pool: &MySqlPool,
    robot_part_id: i64,
) -> Result<Option<RobotPartRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
                rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage \
         FROM RobotPart \
         WHERE id = ?",
    )
    .bind(robot_part_id)
    .fetch_optional(pool)
    .await?;

    row.map(robot_part_record).transpose()
}
