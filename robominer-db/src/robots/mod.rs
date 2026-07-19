use crate::RobotPartRecord;

#[derive(Debug, Clone)]
struct RobotUpdateState {
    id: i64,
    user_id: i64,
    source_code: String,
    ore_container_id: Option<i64>,
    mining_unit_id: Option<i64>,
    battery_id: Option<i64>,
    memory_module_id: Option<i64>,
    cpu_id: Option<i64>,
    engine_id: Option<i64>,
    ore_scanner_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct PendingRobotUpdateState {
    source_code: String,
    ore_container_id: Option<i64>,
    mining_unit_id: Option<i64>,
    battery_id: Option<i64>,
    memory_module_id: Option<i64>,
    cpu_id: Option<i64>,
    engine_id: Option<i64>,
    ore_scanner_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct ProgramSourceUpdateState {
    source_code: Option<String>,
    verified: bool,
    compiled_size: i32,
}

#[derive(Debug, Clone)]
pub struct RequestedRobotParts {
    pub ore_container: RobotPartRecord,
    pub mining_unit: RobotPartRecord,
    pub battery: RobotPartRecord,
    pub memory_module: RobotPartRecord,
    pub cpu: RobotPartRecord,
    pub engine: RobotPartRecord,
    pub ore_scanner: RobotPartRecord,
}

#[derive(Debug, Clone, Copy)]
pub struct RobotParameters {
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
}

mod parameters;
mod read;
mod write;

pub(crate) use parameters::robot_is_recharging;
pub use parameters::robot_parameters_for_parts;
pub use read::*;
pub use write::*;
