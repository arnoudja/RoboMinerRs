use crate::ast::{AreaProperty, ExecutableAction, RobotProperty};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RobotProperties {
    pub forward_speed: f64,
    pub backward_speed: f64,
    pub rotate_speed: f64,
    pub scan_time: f64,
    pub scan_distance: f64,
    pub ore_cap: f64,
    pub max_turns: f64,
    pub mining_speed: f64,
    pub cpu_speed: f64,
    pub orientation: f64,
    pub x_pos: f64,
    pub y_pos: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AreaProperties {
    /// Spawn-local x of the opposite corner (`size_x - robot_size`).
    pub size_x: f64,
    /// Spawn-local y of the opposite corner (`size_y - robot_size`).
    pub size_y: f64,
    pub container_tax: i32,
    pub depot_tax: i32,
    pub starting_ore_a: i32,
    pub starting_ore_b: i32,
    pub starting_ore_c: i32,
    pub mining_turns: i32,
    pub ore_target: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {
    pub time_left: i32,
    pub ore: [i32; crate::MAX_ORE_TYPES],
    pub depot: [i32; crate::MAX_ORE_TYPES],
    pub depot_capacity: [i32; crate::MAX_ORE_TYPES],
    pub action_result: Option<f64>,
    pub scan_time: i32,
    pub scan_started: bool,
    pub scan_complete: bool,
    pub scan_distance: f64,
    pub scan_ore_type: f64,
    pub robot: RobotProperties,
    pub area: AreaProperties,
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
            depot: [0; crate::MAX_ORE_TYPES],
            depot_capacity: [0; crate::MAX_ORE_TYPES],
            action_result,
            scan_time: 0,
            scan_started: false,
            scan_complete: false,
            scan_distance: -1.0,
            scan_ore_type: 0.0,
            robot: RobotProperties::default(),
            area: AreaProperties::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramStep {
    Cpu,
    Action(ExecutableAction),
    Done,
    /// Internal runner invariant failed (stack underflow, missing frame, etc.).
    /// Callers must halt this runner without restarting it, so a corrupted or
    /// buggy executable cannot livelock the simulation.
    Fault,
}

impl AreaProperty {
    pub fn value(self, area: &AreaProperties) -> f64 {
        match self {
            Self::SizeX => area.size_x,
            Self::SizeY => area.size_y,
            Self::ContainerTax => area.container_tax as f64,
            Self::DepotTax => area.depot_tax as f64,
            Self::StartingOreA => area.starting_ore_a as f64,
            Self::StartingOreB => area.starting_ore_b as f64,
            Self::StartingOreC => area.starting_ore_c as f64,
            Self::MiningTurns => area.mining_turns as f64,
            Self::OreTarget => area.ore_target as f64,
        }
    }
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
            Self::MaxTurns => robot.max_turns,
            Self::MiningSpeed => robot.mining_speed,
            Self::CpuSpeed => robot.cpu_speed,
            Self::Orientation => robot.orientation,
            Self::XPos => robot.x_pos,
            Self::YPos => robot.y_pos,
            Self::OreStored
            | Self::OreStoredA
            | Self::OreStoredB
            | Self::OreStoredC
            | Self::DepotSizeA
            | Self::DepotSizeB
            | Self::DepotSizeC
            | Self::DepotStoredA
            | Self::DepotStoredB
            | Self::DepotStoredC => {
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

    pub fn depot_value(
        self,
        depot: &[i32; crate::MAX_ORE_TYPES],
        depot_capacity: &[i32; crate::MAX_ORE_TYPES],
    ) -> Option<f64> {
        Some(match self {
            Self::DepotSizeA => depot_capacity.first().copied().unwrap_or(0) as f64,
            Self::DepotSizeB => depot_capacity.get(1).copied().unwrap_or(0) as f64,
            Self::DepotSizeC => depot_capacity.get(2).copied().unwrap_or(0) as f64,
            Self::DepotStoredA => depot.first().copied().unwrap_or(0) as f64,
            Self::DepotStoredB => depot.get(1).copied().unwrap_or(0) as f64,
            Self::DepotStoredC => depot.get(2).copied().unwrap_or(0) as f64,
            _ => return None,
        })
    }
}
