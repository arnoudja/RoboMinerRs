//! Robot program compiler and interpreter.
//!
//! Multi-cycle move/rotate coordination with the simulator is documented in
//! [`pending_action_protocol`].

mod compile;
pub mod motion;
pub mod pending_action_protocol;
mod pending_await;
mod pending_physical_action;
mod pose;
mod runner;
mod types;

#[cfg(test)]
mod tests;

pub use compile::{
    clear_compile_cache, compatibility_fixture_source, compatibility_fixtures, compile_cache_stats,
    compile_executable_source, compile_source, verify_source,
};
pub use pending_await::{ActionAwaitKind, await_kind};
pub use pose::{rally_map_position, rally_robot_pose};
pub use runner::ExecutableRunner;
pub use types::{
    CompatibilityFixture, CompileError, ExecutableAction, ExecutableProgram, ExecutionContext,
    ProgramStep, RobotProperties, Verification,
};
