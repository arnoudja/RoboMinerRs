#[derive(Debug, Clone, PartialEq)]
pub struct RobotConfigStateRecord {
    pub robot_id: i64,
    pub robot_name: String,
    pub program_source_id: i64,
    pub ore_container_id: i64,
    pub ore_container_name: String,
    pub mining_unit_id: i64,
    pub mining_unit_name: String,
    pub battery_id: i64,
    pub battery_name: String,
    pub battery_capacity: i32,
    pub memory_module_id: i64,
    pub memory_module_name: String,
    pub cpu_id: i64,
    pub cpu_name: String,
    pub engine_id: i64,
    pub engine_name: String,
    pub engine_forward_capacity: i32,
    pub ore_scanner_id: i64,
    pub ore_scanner_name: String,
    pub recharge_time: i32,
    pub max_ore: i32,
    pub mining_speed: i32,
    pub max_turns: i32,
    pub memory_size: i32,
    pub cpu_speed: i32,
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: i32,
    pub robot_size: f64,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub change_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotConfigPartAssetStateRecord {
    pub type_id: i64,
    pub robot_part_id: i64,
    pub part_name: String,
    pub ore_capacity: i32,
    pub mining_capacity: i32,
    pub battery_capacity: i32,
    pub memory_capacity: i32,
    pub cpu_capacity: i32,
    pub forward_capacity: i32,
    pub scan_distance: i32,
    pub unassigned: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RobotRecord {
    pub id: i64,
    pub user_id: i64,
    pub robot_name: String,
    pub source_code: String,
    pub program_source_id: Option<i64>,
    pub ore_container_id: Option<i64>,
    pub mining_unit_id: Option<i64>,
    pub battery_id: Option<i64>,
    pub memory_module_id: Option<i64>,
    pub cpu_id: Option<i64>,
    pub engine_id: Option<i64>,
    pub ore_scanner_id: Option<i64>,
    pub recharge_time: i32,
    pub max_ore: i32,
    pub mining_speed: i32,
    pub max_turns: i32,
    pub memory_size: i32,
    pub cpu_speed: i32,
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: i32,
    pub robot_size: f64,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub total_mining_runs: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AIRobotRecord {
    pub id: i64,
    pub robot_name: String,
    pub source_code: String,
    pub max_ore: i32,
    pub mining_speed: i32,
    pub max_turns: i32,
    pub cpu_speed: i32,
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: i32,
    pub robot_size: f64,
    pub scan_time: i32,
    pub scan_distance: i32,
    pub depot_size: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RobotMiningAreaScoreRecord {
    pub robot_id: i64,
    pub mining_area_id: i64,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotStatsHeaderRecord {
    pub robot_id: i64,
    pub robot_name: String,
    pub username: String,
    pub total_mining_runs: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotLifetimeOreStatRecord {
    pub ore_id: i64,
    pub ore_name: String,
    pub amount: i32,
    pub tax: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RobotMiningAreaStatRecord {
    pub mining_area_id: i64,
    pub area_name: String,
    pub total_runs: i32,
    pub score: f64,
}
