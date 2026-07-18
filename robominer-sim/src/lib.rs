//! Robot mining simulation.

mod action_mapping;
mod animation;
mod ground;
mod physics;
mod position;
mod robot;
mod score;
mod simulation;

#[cfg(test)]
mod tests;

pub use animation::{ANIMATION_PAYLOAD_VERSION, OreAnimationData, is_legacy_javascript_result_data};
pub use ground::{Ground, GroundUnit, ScanResult, ScanSnapshot};
pub use position::Position;
pub use robot::{ROBOT_ACTION_TYPE_SCAN, Robot, RobotAction, RobotSpec, ScriptedRobot};
pub use score::{calculate_score, ore_amounts};
pub use simulation::Simulation;

pub const MAX_ORE_TYPES: usize = 10;
