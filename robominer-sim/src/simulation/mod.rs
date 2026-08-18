mod action_step;
mod collisions;
mod program_bridge;
#[cfg(test)]
mod test_support;

use robominer_program::ExecutableRunner;
use robominer_program::LANGUAGE_ORE_SLOTS;
use robominer_program::motion::is_zero_motion;

use crate::OreAnimationData;
use crate::action_mapping::PendingSimMotionChunk;
use crate::action_mapping::status_for_pending_wait;
use crate::animation::{AnimationRecorder, RecordedCpuStep, RobotCycleStatus};
use crate::ground::{Ground, ScanState};
use crate::physics::{ActionResult, apply_mining};
use crate::position::Position;
use crate::robot::{ActionSource, ROBOT_ACTION_TYPE_SCAN, Robot, RobotAction, ScriptedRobot};

/// Area-level values exposed to robot programs as `area.*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimulationAreaConfig {
    pub container_tax: i32,
    pub depot_tax: i32,
    pub ore_target: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct AreaSnapshot {
    container_tax: i32,
    depot_tax: i32,
    starting_ore: [i32; LANGUAGE_ORE_SLOTS],
    robot_turns: i32,
    ore_target: i32,
}

fn starting_ore_from_ground(ground: &Ground) -> [i32; LANGUAGE_ORE_SLOTS] {
    let mut totals = [0; LANGUAGE_ORE_SLOTS];
    for x in 0..ground.size_x() {
        for y in 0..ground.size_y() {
            let unit = ground.at(x, y);
            for (slot, total) in totals.iter_mut().enumerate() {
                *total += unit.ore_at(slot);
            }
        }
    }
    totals
}

fn animation_action_index(action: RobotAction, robot: &Robot, scanned_this_cycle: bool) -> u8 {
    let scan_busy = scanned_this_cycle || matches!(robot.scan_state, ScanState::Scanning { .. });
    if scan_busy && matches!(action, RobotAction::Wait) {
        ROBOT_ACTION_TYPE_SCAN as u8
    } else {
        action.action_index() as u8
    }
}

/// Carry forward the last known statement highlight (and refreshed locals) for
/// pending multi-turn motion or battery-idle turns that produce no new CPU steps.
fn sticky_cpu_highlight(
    previous: &RecordedCpuStep,
    runner: Option<&ExecutableRunner>,
) -> RecordedCpuStep {
    let mut sticky = previous.clone();
    sticky.result = None;
    if let Some(runner) = runner {
        sticky.variables = runner.runtime_variables_snapshot();
    }
    sticky
}

pub struct Simulation {
    ground: Ground,
    ore_ids: Vec<i64>,
    total_moves: i32,
    area: AreaSnapshot,
    robots: Vec<Robot>,
    action_sources: Vec<ActionSource>,
    action_results: Vec<Option<f64>>,
    action_result_expected: Vec<bool>,
    /// Per-cycle move/rotate chunks; see `robominer_program::pending_action_protocol`.
    pending_sim_motion_chunks: Vec<Option<PendingSimMotionChunk>>,
    /// Last CPU step with a known span per robot, reused for sticky pending-motion cycles.
    last_cpu_highlight: Vec<Option<RecordedCpuStep>>,
    /// Only seed `last_cpu_highlight` from `cycle_cpu_steps[floor..]` after a Done restart.
    cpu_highlight_seed_floor: Vec<usize>,
    time: i32,
    animation: Option<AnimationRecorder>,
}

impl Simulation {
    /// When a cycle produced no new CPU steps, optionally emit a sticky highlight.
    /// Prefer pending multi-cycle motion chunks; otherwise match battery/idle by source line.
    fn maybe_push_sticky_cpu(
        &mut self,
        index: usize,
        cycle_cpu_steps: &mut [Vec<RecordedCpuStep>],
        cycle_source_lines: &mut [Option<u16>],
    ) {
        if !cycle_cpu_steps[index].is_empty() {
            return;
        }
        let seed = if self.pending_sim_motion_chunks[index].is_some() {
            self.last_cpu_highlight[index].clone()
        } else if let Some(line) = cycle_source_lines[index] {
            self.last_cpu_highlight[index]
                .clone()
                .filter(|step| step.line == line)
        } else {
            None
        };
        if let Some(previous) = seed {
            cycle_cpu_steps[index]
                .push(sticky_cpu_highlight(&previous, self.program_runner(index)));
            cycle_source_lines[index] = None;
        }
    }

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
                robot_turns: total_moves,
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

    fn program_entry_source_line(&self, robot_index: usize) -> Option<u16> {
        match &self.action_sources[robot_index] {
            ActionSource::Program { program, .. } => program.entry_source_line(),
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
        let mut cycle_source_lines = vec![None; self.robots.len()];
        let mut cycle_statuses = vec![None; self.robots.len()];
        let mut cycle_cpu_steps = vec![Vec::new(); self.robots.len()];

        if self.time > 0 {
            for (index, pending_result) in pending_results.iter_mut().enumerate() {
                if self.robots[index].spec.max_turns >= self.time {
                    let scan_before = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
                    let (action, status, cpu_steps) = self.next_robot_action(index);
                    let scan_after = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
                    let action_index = animation_action_index(
                        action,
                        &self.robots[index],
                        scan_after > scan_before,
                    );
                    cycle_actions[index] = Some(action_index);
                    cycle_statuses[index] = if action_index == ROBOT_ACTION_TYPE_SCAN as u8 {
                        Some(RobotCycleStatus::Scan)
                    } else if matches!(action, RobotAction::Wait) {
                        status.or(Some(RobotCycleStatus::Wait))
                    } else {
                        status
                    };
                    cycle_source_lines[index] = self
                        .program_runner(index)
                        .and_then(ExecutableRunner::current_source_line);
                    cycle_cpu_steps[index] = cpu_steps;
                    if !cycle_cpu_steps[index].is_empty() {
                        // Prefer `cpu` spans; omit redundant sticky `l` for this cycle.
                        cycle_source_lines[index] = None;
                        let floor = self.cpu_highlight_seed_floor[index];
                        self.cpu_highlight_seed_floor[index] = 0;
                        // Sticky pin from pending-motion assign wins over cycle reseed
                        // (e.g. line-only Action issuer must not lose to an earlier column span).
                        if self.pending_sim_motion_chunks[index].is_none() {
                            let seed_steps = cycle_cpu_steps[index].get(floor..).unwrap_or(&[]);
                            // Prefer column spans; fall back to line-only (e.g. GP) so sticky
                            // pending-motion cycles still have a highlight seed.
                            if let Some(step) = seed_steps
                                .iter()
                                .rev()
                                .find(|step| step.has_columns())
                                .or_else(|| seed_steps.last())
                            {
                                self.last_cpu_highlight[index] = Some(step.clone());
                            }
                        }
                    } else {
                        self.cpu_highlight_seed_floor[index] = 0;
                        self.maybe_push_sticky_cpu(
                            index,
                            &mut cycle_cpu_steps,
                            &mut cycle_source_lines,
                        );
                    }
                    *pending_result = self.process_robot_action(index, action);
                } else {
                    // Battery expired: force-complete in-flight motion so the runner is
                    // not left awaiting a result that will never arrive.
                    if self.pending_sim_motion_chunks[index].is_some() {
                        self.record_action_result(index, ActionResult::None);
                    } else {
                        self.action_results[index] = None;
                    }
                    self.action_result_expected[index] = false;
                    if let ActionSource::Program { runner, .. } = &mut self.action_sources[index] {
                        runner.clear_pending_action_handshake();
                    }
                    self.cpu_highlight_seed_floor[index] = 0;
                    cycle_statuses[index] = Some(RobotCycleStatus::Battery);
                    // Keep the last statement highlight after the battery expires.
                    cycle_source_lines[index] = self
                        .program_runner(index)
                        .and_then(ExecutableRunner::current_source_line);
                    self.maybe_push_sticky_cpu(
                        index,
                        &mut cycle_cpu_steps,
                        &mut cycle_source_lines,
                    );
                }
            }

            // Travel after wall clipping, before robot-robot collisions.
            let mut pre_collision_travel = vec![None; self.robots.len()];
            for (index, result) in pending_results.iter().enumerate() {
                if matches!(result, ActionResult::Move { .. }) {
                    pre_collision_travel[index] = Some(
                        self.robots[index]
                            .position
                            .distance(&self.robots[index].destination),
                    );
                }
            }

            self.check_collisions();

            for (index, result) in pending_results.iter_mut().enumerate() {
                if let ActionResult::Move { direction } = *result {
                    let distance = self.robots[index]
                        .position
                        .distance(&self.robots[index].destination);
                    if is_zero_motion(distance) {
                        let wall_blocked = pre_collision_travel[index].is_some_and(is_zero_motion);
                        cycle_statuses[index] = Some(if wall_blocked {
                            RobotCycleStatus::Wall
                        } else {
                            RobotCycleStatus::Robot
                        });
                    }
                    *result = ActionResult::Value(distance * direction);
                }
            }
        } else {
            for (index, line) in cycle_source_lines.iter_mut().enumerate() {
                *line = self.program_entry_source_line(index);
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
                animation.record_robot_step(
                    index,
                    robot,
                    cycle_actions[index],
                    cycle_source_lines[index],
                    cycle_statuses[index],
                    std::mem::take(&mut cycle_cpu_steps[index]),
                );
            }
        }

        for (index, result) in pending_results.into_iter().enumerate() {
            self.record_action_result(index, result);
        }
    }

    fn next_robot_action(
        &mut self,
        robot_index: usize,
    ) -> (RobotAction, Option<RobotCycleStatus>, Vec<RecordedCpuStep>) {
        if let Some(pending) = &self.pending_sim_motion_chunks[robot_index] {
            self.action_result_expected[robot_index] = true;
            let action = pending.next_robot_action(self.robots[robot_index].spec());
            let status = if matches!(action, RobotAction::Wait) {
                Some(status_for_pending_wait(pending))
            } else {
                None
            };
            return (action, status, Vec::new());
        }

        match &mut self.action_sources[robot_index] {
            ActionSource::Actions(actions) => {
                self.action_result_expected[robot_index] = false;
                let action = actions
                    .get((self.time - 1) as usize)
                    .copied()
                    .unwrap_or(RobotAction::Wait);
                let status = if matches!(action, RobotAction::Wait) {
                    Some(RobotCycleStatus::Wait)
                } else {
                    None
                };
                (action, status, Vec::new())
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
