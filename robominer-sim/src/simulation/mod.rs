mod action_step;
mod collisions;
mod helpers;
mod program_bridge;
mod step;
#[cfg(test)]
mod test_support;

use robominer_program::ExecutableRunner;

use crate::action_mapping::PendingSimMotionChunk;
use crate::animation::{AnimationRecorder, RecordedCpuStep};
use crate::ground::Ground;
use crate::robot::{ActionSource, Robot, ScriptedRobot};

pub use helpers::SimulationAreaConfig;

use helpers::{AreaSnapshot, starting_ore_from_ground};

pub struct Simulation {
    pub(super) ground: Ground,
    pub(super) ore_ids: Vec<i64>,
    pub(super) total_moves: i32,
    pub(super) area: AreaSnapshot,
    pub(super) robots: Vec<Robot>,
    pub(super) action_sources: Vec<ActionSource>,
    pub(super) action_results: Vec<Option<f64>>,
    pub(super) action_result_expected: Vec<bool>,
    /// Per-cycle move/rotate chunks; see `robominer_program::pending_action_protocol`.
    pub(super) pending_sim_motion_chunks: Vec<Option<PendingSimMotionChunk>>,
    /// Last CPU step with a known span per robot, reused for sticky pending-motion cycles.
    pub(super) last_cpu_highlight: Vec<Option<RecordedCpuStep>>,
    /// Only seed `last_cpu_highlight` from `cycle_cpu_steps[floor..]` after a Done restart.
    pub(super) cpu_highlight_seed_floor: Vec<usize>,
    pub(super) time: i32,
    pub(super) animation: Option<AnimationRecorder>,
}

impl Simulation {
    pub fn new(ground: Ground, total_moves: i32, robots: Vec<ScriptedRobot>) -> Self {
        Self::new_with_ore_ids(ground, total_moves, robots, Vec::new())
    }

    pub fn new_with_ore_ids(
        ground: Ground,
        total_moves: i32,
        robots: Vec<ScriptedRobot>,
        ore_ids: Vec<i64>,
    ) -> Self {
        Self::new_with_area(
            ground,
            total_moves,
            robots,
            ore_ids,
            SimulationAreaConfig::default(),
        )
    }

    pub fn new_with_area(
        ground: Ground,
        total_moves: i32,
        robots: Vec<ScriptedRobot>,
        ore_ids: Vec<i64>,
        area: SimulationAreaConfig,
    ) -> Self {
        assert!(total_moves >= 0);
        assert!(!robots.is_empty());
        assert!(robots.len() <= 4);

        let starting_ore = starting_ore_from_ground(&ground);
        let action_sources: Vec<_> = robots
            .iter()
            .map(|robot| robot.action_source.clone())
            .collect();
        let action_results = vec![None; action_sources.len()];
        let action_result_expected = vec![false; action_sources.len()];
        let pending_sim_motion_chunks = vec![None; action_sources.len()];
        let last_cpu_highlight = vec![None; action_sources.len()];
        let cpu_highlight_seed_floor = vec![0; action_sources.len()];
        let robots = robots
            .into_iter()
            .map(|robot| {
                let mut built = Robot::new(robot.spec);
                built.set_depot_capacity(robot.depot_capacity);
                built
            })
            .collect();

        Self {
            ground,
            ore_ids,
            total_moves,
            area: AreaSnapshot {
                container_tax: area.container_tax,
                depot_tax: area.depot_tax,
                starting_ore,
                mining_turns: total_moves,
                ore_target: area.ore_target,
            },
            robots,
            action_sources,
            action_results,
            action_result_expected,
            pending_sim_motion_chunks,
            last_cpu_highlight,
            cpu_highlight_seed_floor,
            time: 0,
            animation: None,
        }
    }

    pub fn run(&mut self) {
        self.run_internal();
    }

    pub fn run_with_animation(&mut self, ore_data: &[crate::OreAnimationData]) -> String {
        self.animation = Some(AnimationRecorder::new(self.robots.len()));
        self.run_internal();

        let Some(recorder) = self.animation.take() else {
            return String::new();
        };
        recorder.into_animation_data(&self.ground, &self.robots, ore_data)
    }

    fn run_internal(&mut self) {
        let max_robot_turns = self
            .robots
            .iter()
            .map(|robot| robot.spec.max_turns)
            .max()
            .unwrap_or(0);
        self.total_moves = self.total_moves.min(max_robot_turns);

        self.init_robot_positions();

        if let Some(animation) = &mut self.animation {
            animation.record_initial_ground(&self.ground);
        }

        for time in 0..=self.total_moves {
            self.time = time;
            self.process_step();
        }
    }

    pub fn ground(&self) -> &Ground {
        &self.ground
    }

    pub fn robot(&self, index: usize) -> &Robot {
        &self.robots[index]
    }

    pub fn time(&self) -> i32 {
        self.time
    }

    /// Live program runner for robots driven by an executable program.
    pub fn program_runner(&self, robot_index: usize) -> Option<&ExecutableRunner> {
        match &self.action_sources[robot_index] {
            ActionSource::Program { runner, .. } => Some(runner.as_ref()),
            _ => None,
        }
    }

    fn program_entry_source_line(&self, robot_index: usize) -> Option<u16> {
        match &self.action_sources[robot_index] {
            ActionSource::Program { program, .. } => program.entry_source_line(),
            _ => None,
        }
    }
}
