#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Robot mining simulation.

mod action_mapping;
mod animation;
mod animation_payload;
mod ground;
mod physics;
mod position;
mod robot;
mod score;
mod simulation;

#[cfg(test)]
mod tests;

pub use animation::{
    ANIMATION_PAYLOAD_VERSION, OreAnimationData, RecordedCpuStep, is_legacy_javascript_result_data,
};
pub use animation_payload::{
    AnimationCpuStep, AnimationCpuStepResult, AnimationCpuStepResultKind, AnimationGround,
    AnimationGroundChange, AnimationGroundPosition, AnimationLocation, AnimationOreType,
    AnimationPayload, AnimationRobot, AnimationRobots,
};
pub use ground::{Ground, GroundUnit, ScanResult, ScanSnapshot, ore_heap_estimated_total};
pub use position::Position;
pub use robot::{ROBOT_ACTION_TYPE_SCAN, Robot, RobotAction, RobotSpec, ScriptedRobot};
pub use score::{
    SCORE_TIER_COUNT, ScoreBreakdown, ScoreTierBreakdown, calculate_score, ore_amounts,
    score_breakdown,
};
pub use simulation::{Simulation, SimulationAreaConfig};

pub use robominer_program::MAX_ORE_TYPES;
