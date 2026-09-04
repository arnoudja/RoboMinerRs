//! Catalog reads and well-known seed identifiers shared by web and engine.

/// `RobotPartType.id` values from `resources/database/gameData.sql`.
pub mod part_type_id {
    pub const ORE_CONTAINER: i64 = 1;
    pub const MINING_UNIT: i64 = 2;
    pub const BATTERY: i64 = 3;
    pub const MEMORY_MODULE: i64 = 4;
    pub const CPU: i64 = 5;
    pub const ENGINE: i64 = 6;
    pub const ORE_SCANNER: i64 = 7;
}

/// Default starter `RobotPart.id` values from `resources/database/gameData.sql`
/// (one part per type, in `part_type_id` order).
pub const DEFAULT_PART_IDS: [i64; 7] = [101, 201, 301, 401, 501, 601, 701];

use sqlx::MySqlPool;

use crate::{
    OreRecord, RobotPartRecord, RobotPartTypeRecord, ShopRobotPartCatalogRecord,
    ShopRobotPartCostRecord,
};

#[derive(sqlx::FromRow)]
pub(crate) struct RobotPartRow {
    id: i64,
    #[sqlx(rename = "typeId")]
    type_id: i64,
    #[sqlx(rename = "tierId")]
    tier_id: Option<i64>,
    #[sqlx(rename = "partName")]
    part_name: String,
    #[sqlx(rename = "orePriceId")]
    ore_price_id: i64,
    #[sqlx(rename = "oreCapacity")]
    ore_capacity: i32,
    #[sqlx(rename = "miningCapacity")]
    mining_capacity: i32,
    #[sqlx(rename = "batteryCapacity")]
    battery_capacity: i32,
    #[sqlx(rename = "memoryCapacity")]
    memory_capacity: i32,
    #[sqlx(rename = "cpuCapacity")]
    cpu_capacity: i32,
    #[sqlx(rename = "forwardCapacity")]
    forward_capacity: i32,
    #[sqlx(rename = "backwardCapacity")]
    backward_capacity: i32,
    #[sqlx(rename = "rotateCapacity")]
    rotate_capacity: i32,
    #[sqlx(rename = "rechargeTime")]
    recharge_time: i32,
    #[sqlx(rename = "scanTime")]
    scan_time: i32,
    #[sqlx(rename = "scanDistance")]
    scan_distance: i32,
    weight: i32,
    volume: i32,
    #[sqlx(rename = "powerUsage")]
    power_usage: i32,
}

impl From<RobotPartRow> for RobotPartRecord {
    fn from(row: RobotPartRow) -> Self {
        Self {
            id: row.id,
            type_id: row.type_id,
            tier_id: row.tier_id,
            part_name: row.part_name,
            ore_price_id: row.ore_price_id,
            ore_capacity: row.ore_capacity,
            mining_capacity: row.mining_capacity,
            battery_capacity: row.battery_capacity,
            memory_capacity: row.memory_capacity,
            cpu_capacity: row.cpu_capacity,
            forward_capacity: row.forward_capacity,
            backward_capacity: row.backward_capacity,
            rotate_capacity: row.rotate_capacity,
            recharge_time: row.recharge_time,
            scan_time: row.scan_time,
            scan_distance: row.scan_distance,
            weight: row.weight,
            volume: row.volume,
            power_usage: row.power_usage,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ShopRobotPartCatalogRow {
    #[sqlx(rename = "id")]
    robot_part_id: i64,
    #[sqlx(rename = "typeId")]
    type_id: i64,
    #[sqlx(rename = "tierId")]
    tier_id: i64,
    #[sqlx(rename = "oreName")]
    tier_name: String,
    #[sqlx(rename = "partName")]
    part_name: String,
    #[sqlx(rename = "oreCapacity")]
    ore_capacity: i32,
    #[sqlx(rename = "miningCapacity")]
    mining_capacity: i32,
    #[sqlx(rename = "batteryCapacity")]
    battery_capacity: i32,
    #[sqlx(rename = "memoryCapacity")]
    memory_capacity: i32,
    #[sqlx(rename = "cpuCapacity")]
    cpu_capacity: i32,
    #[sqlx(rename = "forwardCapacity")]
    forward_capacity: i32,
    #[sqlx(rename = "backwardCapacity")]
    backward_capacity: i32,
    #[sqlx(rename = "rotateCapacity")]
    rotate_capacity: i32,
    #[sqlx(rename = "rechargeTime")]
    recharge_time: i32,
    #[sqlx(rename = "scanTime")]
    scan_time: i32,
    #[sqlx(rename = "scanDistance")]
    scan_distance: i32,
    weight: i32,
    volume: i32,
    #[sqlx(rename = "powerUsage")]
    power_usage: i32,
}

impl From<ShopRobotPartCatalogRow> for ShopRobotPartCatalogRecord {
    fn from(row: ShopRobotPartCatalogRow) -> Self {
        Self {
            robot_part_id: row.robot_part_id,
            type_id: row.type_id,
            tier_id: row.tier_id,
            tier_name: row.tier_name,
            part_name: row.part_name,
            ore_capacity: row.ore_capacity,
            mining_capacity: row.mining_capacity,
            battery_capacity: row.battery_capacity,
            memory_capacity: row.memory_capacity,
            cpu_capacity: row.cpu_capacity,
            forward_capacity: row.forward_capacity,
            backward_capacity: row.backward_capacity,
            rotate_capacity: row.rotate_capacity,
            recharge_time: row.recharge_time,
            scan_time: row.scan_time,
            scan_distance: row.scan_distance,
            weight: row.weight,
            volume: row.volume,
            power_usage: row.power_usage,
        }
    }
}

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
    sqlx::query_as::<_, ShopRobotPartCatalogRow>(
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
    .await
    .map(|rows| {
        rows.into_iter()
            .map(ShopRobotPartCatalogRecord::from)
            .collect()
    })
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
    sqlx::query_as::<_, RobotPartRow>(
        "SELECT id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
                rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage \
         FROM RobotPart \
         WHERE id = ?",
    )
    .bind(robot_part_id)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(RobotPartRecord::from))
}

pub async fn list_robot_parts(pool: &MySqlPool) -> Result<Vec<RobotPartRecord>, sqlx::Error> {
    sqlx::query_as::<_, RobotPartRow>(
        "SELECT id, typeId, tierId, partName, orePriceId, oreCapacity, miningCapacity, \
                batteryCapacity, memoryCapacity, cpuCapacity, forwardCapacity, backwardCapacity, \
                rotateCapacity, rechargeTime, scanTime, scanDistance, weight, volume, powerUsage \
         FROM RobotPart \
         ORDER BY typeId, id",
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(RobotPartRecord::from).collect())
}
