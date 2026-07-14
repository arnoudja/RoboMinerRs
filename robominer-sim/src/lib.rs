//! Robot mining simulation.

mod action_mapping;
mod animation;
mod ground;
mod physics;
mod score;
mod position;
mod robot;
mod simulation;

#[cfg(test)]
mod tests;

pub use animation::OreAnimationData;
pub use ground::{Ground, GroundUnit, ScanResult, ScanSnapshot};
pub use score::{calculate_score, ore_amounts};
pub use position::Position;
pub use robot::{Robot, RobotAction, RobotSpec, ScriptedRobot, ROBOT_ACTION_TYPE_SCAN};
pub use simulation::Simulation;

pub const MAX_ORE_TYPES: usize = 10;
