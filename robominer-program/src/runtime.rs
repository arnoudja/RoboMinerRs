use crate::ast::{ExecutableAction, RobotProperty};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RobotProperties {
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: f64,
    pub scan_time: f64,
    pub scan_distance: f64,
    pub ore_cap: f64,
    pub max_cycles: f64,
    pub mining_speed: f64,
    pub cpu_speed: f64,
    pub orientation: f64,
    pub x_pos: f64,
    pub y_pos: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {
    pub time_left: i32,
    pub ore: [i32; crate::MAX_ORE_TYPES],
    pub action_result: Option<f64>,
    pub scan_time: i32,
    pub scan_started: bool,
    pub scan_complete: bool,
    pub scan_distance: f64,
    pub scan_ore_type: f64,
    pub robot: RobotProperties,
}

impl ExecutionContext {
    pub fn from_runtime(
        time_left: i32,
        ore: [i32; crate::MAX_ORE_TYPES],
        action_result: Option<f64>,
    ) -> Self {
        Self {
            time_left,
            ore,
            action_result,
            scan_time: 0,
            scan_started: false,
            scan_complete: false,
            scan_distance: -1.0,
            scan_ore_type: 0.0,
            robot: RobotProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramStep {
    Cpu,
    Action(ExecutableAction),
    Done,
}

impl RobotProperty {
    pub fn value(self, robot: &RobotProperties) -> Option<f64> {
        Some(match self {
            Self::ForwardSpeed => robot.forward_speed,
            Self::BackwardSpeed => robot.backward_speed,
            Self::RotateSpeed => robot.rotate_speed,
            Self::ScanTime => robot.scan_time,
            Self::ScanDistance => robot.scan_distance,
            Self::OreCap => robot.ore_cap,
            Self::MaxCycles => robot.max_cycles,
            Self::MiningSpeed => robot.mining_speed,
            Self::CpuSpeed => robot.cpu_speed,
            Self::Orientation => robot.orientation,
            Self::XPos => robot.x_pos,
            Self::YPos => robot.y_pos,
            Self::OreStored | Self::OreStoredA | Self::OreStoredB | Self::OreStoredC => {
                return None;
            }
        })
    }

    pub fn stored_ore_value(self, ore: &[i32; crate::MAX_ORE_TYPES]) -> Option<f64> {
        Some(match self {
            Self::OreStored => ore.iter().sum::<i32>() as f64,
            Self::OreStoredA => ore.first().copied().unwrap_or(0) as f64,
            Self::OreStoredB => ore.get(1).copied().unwrap_or(0) as f64,
            Self::OreStoredC => ore.get(2).copied().unwrap_or(0) as f64,
            _ => return None,
        })
    }
}
