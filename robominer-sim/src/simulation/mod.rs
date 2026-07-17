mod action_step;
mod collisions;
mod program_bridge;
#[cfg(test)]
mod test_support;

use robominer_program::ExecutableRunner;

use crate::OreAnimationData;
use crate::action_mapping::PendingExpressionAction;
use crate::animation::AnimationRecorder;
use crate::ground::Ground;
use crate::physics::{ActionResult, apply_mining};
use crate::position::Position;
use crate::robot::{ActionSource, ROBOT_ACTION_TYPE_SCAN, Robot, RobotAction, ScriptedRobot};

fn animation_action_index(action: RobotAction, scanned_this_cycle: bool) -> u8 {
    if scanned_this_cycle && matches!(action, RobotAction::Wait) {
        ROBOT_ACTION_TYPE_SCAN as u8
    } else {
        action.action_index() as u8
    }
}

pub struct Simulation {
    ground: Ground,
    ore_ids: Vec<i64>,
    total_moves: i32,
    robots: Vec<Robot>,
    action_sources: Vec<ActionSource>,
    action_results: Vec<Option<f64>>,
    action_result_expected: Vec<bool>,
    /// Per-cycle move/rotate chunks; see `robominer_program::pending_action_protocol`.
    pending_expression_actions: Vec<Option<PendingExpressionAction>>,
    time: i32,
    animation: Option<AnimationRecorder>,
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
        assert!(total_moves >= 0);
        assert!(!robots.is_empty());
        assert!(robots.len() <= 4);

        let action_sources: Vec<_> = robots
            .iter()
            .map(|robot| robot.action_source.clone())
            .collect();
        let action_results = vec![None; action_sources.len()];
        let action_result_expected = vec![false; action_sources.len()];
        let pending_expression_actions = vec![None; action_sources.len()];
        let robots = robots
            .into_iter()
            .map(|robot| Robot::new(robot.spec))
            .collect();

        Self {
            ground,
            ore_ids,
            total_moves,
            robots,
            action_sources,
            action_results,
            action_result_expected,
            pending_expression_actions,
            time: 0,
            animation: None,
        }
    }

    pub fn run(&mut self) {
        self.run_internal();
    }

    pub fn run_with_animation(&mut self, ore_data: &[OreAnimationData]) -> String {
        self.animation = Some(AnimationRecorder::new(self.robots.len()));
        self.run_internal();

        self.animation
            .take()
            .expect("animation recorder should be available")
            .into_animation_data(&self.ground, &self.robots, ore_data)
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

    fn init_robot_positions(&mut self) {
        let size_x = self.ground.size_x() as f64;
        let size_y = self.ground.size_y() as f64;

        for (index, robot) in self.robots.iter_mut().enumerate() {
            robot.min_x = robot.spec.robot_size / 2.0 - 0.5;
            robot.min_y = robot.spec.robot_size / 2.0 - 0.5;
            robot.max_x = size_x - robot.spec.robot_size / 2.0 - 0.5;
            robot.max_y = size_y - robot.spec.robot_size / 2.0 - 0.5;

            match index {
                0 => robot.position = Position::new(robot.min_x, robot.min_y, 45),
                1 => robot.position = Position::new(robot.min_x, robot.max_y, 315),
                2 => robot.position = Position::new(robot.max_x, robot.min_y, 135),
                3 => robot.position = Position::new(robot.max_x, robot.max_y, 225),
                _ => unreachable!("simulation supports at most four robots"),
            }

            let center = robot.center_position();
            robot.initial_center_x = center.x;
            robot.initial_center_y = center.y;
            robot.initial_orientation = center.orientation;
        }
    }

    fn process_step(&mut self) {
        for robot in &mut self.robots {
            robot.prepare_for_action(self.time, self.total_moves);
        }

        let mut pending_results = vec![ActionResult::None; self.robots.len()];
        let mut cycle_actions = vec![None; self.robots.len()];

        if self.time > 0 {
            for (index, pending_result) in pending_results.iter_mut().enumerate() {
                if self.robots[index].spec.max_turns >= self.time {
                    let scan_before = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
                    let action = self.next_robot_action(index);
                    let scan_after = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
                    cycle_actions[index] =
                        Some(animation_action_index(action, scan_after > scan_before));
                    *pending_result = self.process_robot_action(index, action);
                } else {
                    self.action_results[index] = None;
                    self.action_result_expected[index] = false;
                    self.pending_expression_actions[index] = None;
                }
            }

            self.check_collisions();

            for (index, result) in pending_results.iter_mut().enumerate() {
                if let ActionResult::Move { direction } = *result {
                    let distance = self.robots[index]
                        .position
                        .distance(&self.robots[index].destination);
                    *result = ActionResult::Value(distance * direction);
                }
            }
        }

        let mut ground_changes = Vec::new();

        for robot in &mut self.robots {
            robot.position = robot.destination;
            robot.apply_rotation();
            if let Some(change) = apply_mining(&mut self.ground, robot, self.time) {
                ground_changes.push(change);
            }
        }

        if let Some(animation) = &mut self.animation {
            for change in ground_changes {
                animation.record_ground_change(change.x, change.y, change.time, change.ore);
            }

            for (index, robot) in self.robots.iter().enumerate() {
                animation.record_robot_step(index, robot, cycle_actions[index]);
            }
        }

        for (index, result) in pending_results.into_iter().enumerate() {
            self.record_action_result(index, result);
        }
    }

    fn next_robot_action(&mut self, robot_index: usize) -> RobotAction {
        if let Some(pending) = &self.pending_expression_actions[robot_index] {
            self.action_result_expected[robot_index] = true;
            return pending.next_robot_action(self.robots[robot_index].spec());
        }

        match &mut self.action_sources[robot_index] {
            ActionSource::Actions(actions) => {
                self.action_result_expected[robot_index] = false;
                actions
                    .get((self.time - 1) as usize)
                    .copied()
                    .unwrap_or(RobotAction::Wait)
            }
            ActionSource::Program { .. } => self.run_program_cpu_loop(robot_index),
        }
    }

    fn should_preserve_program_action_result(&self, robot_index: usize) -> bool {
        self.action_results[robot_index].is_some()
            && matches!(
                &self.action_sources[robot_index],
                ActionSource::Program { runner, .. }
                    if runner.pending_scan_start() || runner.awaits_scan_result()
            )
    }
}
