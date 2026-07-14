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
pub(crate) struct RequestedRobotParts {
    ore_container: RobotPartRecord,
    mining_unit: RobotPartRecord,
    battery: RobotPartRecord,
    memory_module: RobotPartRecord,
    cpu: RobotPartRecord,
    engine: RobotPartRecord,
    ore_scanner: RobotPartRecord,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RobotParameters {
    recharge_time: i32,
    max_ore: i32,
    mining_speed: i32,
    max_turns: i32,
    memory_size: i32,
    cpu_speed: i32,
    forward_speed: f64,
    backward_speed: f64,
    rotate_speed: i32,
    robot_size: f64,
    scan_time: i32,
    scan_distance: i32,
}

mod parameters;
mod read;
mod write;

pub(crate) use parameters::robot_is_recharging;
pub use read::*;
pub use write::*;
