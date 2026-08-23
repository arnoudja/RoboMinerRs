#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaRecord {
    pub id: i64,
    pub area_name: String,
    pub ore_price_id: i64,
    pub size_x: i32,
    pub size_y: i32,
    pub max_moves: i32,
    pub mining_time: i32,
    pub tax_rate: i32,
    pub depot_tax_rate: i32,
    pub score_ore_target: i32,
    pub ai_robot_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaOreSupplyRecord {
    pub id: i64,
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub supply: i32,
    pub radius: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningAreaOverviewOreRecord {
    pub ore_id: i64,
    pub ore_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningAreaOverviewAreaRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub total_average_ore_per_run: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningAreaOverviewOreAverageRecord {
    pub mining_area_id: i64,
    pub ore_id: i64,
    pub average_ore_per_run: f64,
}
