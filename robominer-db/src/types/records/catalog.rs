#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotPartTypeRecord {
    pub id: i64,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotPartRecord {
    pub id: i64,
    pub type_id: i64,
    pub tier_id: Option<i64>,
    pub part_name: String,
    pub ore_price_id: i64,
    pub ore_capacity: i32,
    pub mining_capacity: i32,
    pub battery_capacity: i32,
    pub memory_capacity: i32,
    pub cpu_capacity: i32,
    pub forward_capacity: i32,
    pub backward_capacity: i32,
    pub rotate_capacity: i32,
    pub recharge_time: i32,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub weight: i32,
    pub volume: i32,
    pub power_usage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OreRecord {
    pub id: i64,
    pub ore_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartCatalogRecord {
    pub robot_part_id: i64,
    pub type_id: i64,
    pub tier_id: i64,
    pub tier_name: String,
    pub part_name: String,
    pub ore_capacity: i32,
    pub mining_capacity: i32,
    pub battery_capacity: i32,
    pub memory_capacity: i32,
    pub cpu_capacity: i32,
    pub forward_capacity: i32,
    pub backward_capacity: i32,
    pub rotate_capacity: i32,
    pub recharge_time: i32,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub weight: i32,
    pub volume: i32,
    pub power_usage: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartCostRecord {
    pub robot_part_id: i64,
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopRobotPartStateRecord {
    pub robot_part_id: i64,
    pub total_owned: i32,
    pub assigned: i32,
    pub unassigned: i32,
    pub can_buy: bool,
    pub can_sell: bool,
}
