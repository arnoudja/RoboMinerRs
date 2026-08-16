//! Robot program compiler and interpreter.
//!
//! Multi-cycle move/rotate coordination with the simulator is documented in
//! [`pending_action_protocol`].

mod ast;
mod ast_visit;
mod compile;
mod compile_error;
mod cpu_step_result;
pub mod gp;
pub mod motion;
pub mod pending_action_protocol;
mod pending_await;
mod pending_program_motion;
mod pose;
mod runner;
mod runtime;
mod types;
pub mod unparse;

#[cfg(test)]
mod tests;

pub use compile::{
    CompatibilityFixture, Verification, clear_compile_cache, compatibility_fixture_source,
    compatibility_fixtures, compile_cache_stats, compile_executable_source, compile_source,
    verify_source,
};
pub use cpu_step_result::{CpuStepResult, CpuStepResultKind};
pub use gp::{
    RngLike, crossover_programs, mutate_program, recompile_program, seed_program_sources,
};
pub use pending_await::{ActionAwaitKind, await_kind};
pub use pose::{rally_map_position, rally_robot_pose};
pub use runner::ExecutableRunner;
pub use types::{
    AreaProperties, AreaProperty, CompileError, ExecutableAction, ExecutableActionExpression,
    ExecutableExpression, ExecutableExpressionKind, ExecutableProgram, ExecutableStatement,
    ExecutableStatementKind, ExecutionContext, Operator, ProgramStep, RobotProperties,
    RobotProperty, SourceSpan, ValueType, VariableOperator,
};
pub use unparse::unparse_program;

/// Slot capacity for ore arrays in the runner / sim bridge (A… plus reserved slots).
pub const MAX_ORE_TYPES: usize = 10;
/// Language-facing dump/read slots A/B/C (`dump(1|2|3)`, `robot.oreStoredA|B|C`, `robot.depotSizeA|B|C`, `robot.depotStoredA|B|C`).
pub const LANGUAGE_ORE_SLOTS: usize = 3;
