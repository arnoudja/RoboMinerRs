use robominer_program::ExecutableRunner;
use robominer_program::motion::is_zero_motion;

use crate::action_mapping::status_for_pending_wait;
use crate::animation::{RecordedCpuStep, RobotCycleStatus};
use crate::physics::{ActionResult, GroundAnimationChange, apply_mining};
use crate::position::Position;
use crate::robot::{ActionSource, ROBOT_ACTION_TYPE_SCAN, RobotAction};

use super::Simulation;
use super::helpers::{animation_action_index, sticky_cpu_highlight};

/// Per-robot buffers filled while collecting and resolving one simulation cycle.
struct CycleBuffers {
    pending_results: Vec<ActionResult>,
    cycle_actions: Vec<Option<u8>>,
    cycle_source_lines: Vec<Option<u16>>,
    cycle_statuses: Vec<Option<RobotCycleStatus>>,
    cycle_cpu_steps: Vec<Vec<RecordedCpuStep>>,
}

impl CycleBuffers {
    fn new(robot_count: usize) -> Self {
        Self {
            pending_results: vec![ActionResult::None; robot_count],
            cycle_actions: vec![None; robot_count],
            cycle_source_lines: vec![None; robot_count],
            cycle_statuses: vec![None; robot_count],
            cycle_cpu_steps: vec![Vec::new(); robot_count],
        }
    }
}

impl Simulation {
    pub(super) fn init_robot_positions(&mut self) {
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

    pub(super) fn process_step(&mut self) {
        for robot in &mut self.robots {
            robot.prepare_for_action(self.time, self.total_moves);
        }

        let mut buffers = CycleBuffers::new(self.robots.len());

        if self.time > 0 {
            self.collect_pending_actions(&mut buffers);
            self.resolve_move_collisions(&mut buffers);
        } else {
            self.seed_entry_source_lines(&mut buffers.cycle_source_lines);
        }

        let ground_changes = self.apply_positions_and_mining();
        self.record_cycle_animation(
            &buffers.cycle_actions,
            &buffers.cycle_source_lines,
            &buffers.cycle_statuses,
            &mut buffers.cycle_cpu_steps,
            ground_changes,
        );
        self.record_pending_action_results(buffers.pending_results);
    }

    /// Collect each robot's action for this cycle and process it into a pending result.
    fn collect_pending_actions(&mut self, buffers: &mut CycleBuffers) {
        for index in 0..self.robots.len() {
            if self.robots[index].spec.max_turns >= self.time {
                self.collect_active_robot_action(index, buffers);
            } else {
                self.collect_battery_expired_robot(index, buffers);
            }
        }
    }

    fn collect_active_robot_action(&mut self, index: usize, buffers: &mut CycleBuffers) {
        let scan_before = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
        let (action, status, cpu_steps) = self.next_robot_action(index);
        let scan_after = self.robots[index].actions_done()[ROBOT_ACTION_TYPE_SCAN];
        let action_index =
            animation_action_index(action, &self.robots[index], scan_after > scan_before);
        buffers.cycle_actions[index] = Some(action_index);
        buffers.cycle_statuses[index] = if action_index == ROBOT_ACTION_TYPE_SCAN as u8 {
            Some(RobotCycleStatus::Scan)
        } else if matches!(action, RobotAction::Wait) {
            status.or(Some(RobotCycleStatus::Wait))
        } else {
            status
        };
        buffers.cycle_source_lines[index] = self
            .program_runner(index)
            .and_then(ExecutableRunner::current_source_line);
        buffers.cycle_cpu_steps[index] = cpu_steps;
        if !buffers.cycle_cpu_steps[index].is_empty() {
            // Prefer `cpu` spans; omit redundant sticky `l` for this cycle.
            buffers.cycle_source_lines[index] = None;
            let floor = self.cpu_highlight_seed_floor[index];
            self.cpu_highlight_seed_floor[index] = 0;
            // Sticky pin from pending-motion assign wins over cycle reseed
            // (e.g. line-only Action issuer must not lose to an earlier column span).
            if self.pending_sim_motion_chunks[index].is_none() {
                let seed_steps = buffers.cycle_cpu_steps[index].get(floor..).unwrap_or(&[]);
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
                &mut buffers.cycle_cpu_steps,
                &mut buffers.cycle_source_lines,
            );
        }
        buffers.pending_results[index] = self.process_robot_action(index, action);
    }

    fn collect_battery_expired_robot(&mut self, index: usize, buffers: &mut CycleBuffers) {
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
        buffers.cycle_statuses[index] = Some(RobotCycleStatus::Battery);
        // Keep the last statement highlight after the battery expires.
        buffers.cycle_source_lines[index] = self
            .program_runner(index)
            .and_then(ExecutableRunner::current_source_line);
        self.maybe_push_sticky_cpu(
            index,
            &mut buffers.cycle_cpu_steps,
            &mut buffers.cycle_source_lines,
        );
    }

    /// Travel after wall clipping, before robot-robot collisions; then convert moves to values.
    fn resolve_move_collisions(&mut self, buffers: &mut CycleBuffers) {
        let mut pre_collision_travel = vec![None; self.robots.len()];
        for (index, result) in buffers.pending_results.iter().enumerate() {
            if matches!(result, ActionResult::Move { .. }) {
                pre_collision_travel[index] = Some(
                    self.robots[index]
                        .position
                        .distance(&self.robots[index].destination),
                );
            }
        }

        self.check_collisions();

        for (index, result) in buffers.pending_results.iter_mut().enumerate() {
            if let ActionResult::Move { direction } = *result {
                let distance = self.robots[index]
                    .position
                    .distance(&self.robots[index].destination);
                if is_zero_motion(distance) {
                    let wall_blocked = pre_collision_travel[index].is_some_and(is_zero_motion);
                    buffers.cycle_statuses[index] = Some(if wall_blocked {
                        RobotCycleStatus::Wall
                    } else {
                        RobotCycleStatus::Robot
                    });
                }
                *result = ActionResult::Value(distance * direction);
            }
        }
    }

    fn seed_entry_source_lines(&self, cycle_source_lines: &mut [Option<u16>]) {
        for (index, line) in cycle_source_lines.iter_mut().enumerate() {
            *line = self.program_entry_source_line(index);
        }
    }

    fn apply_positions_and_mining(&mut self) -> Vec<GroundAnimationChange> {
        let mut ground_changes = Vec::new();
        for robot in &mut self.robots {
            robot.position = robot.destination;
            robot.apply_rotation();
            if let Some(change) = apply_mining(&mut self.ground, robot, self.time) {
                ground_changes.push(change);
            }
        }
        ground_changes
    }

    fn record_cycle_animation(
        &mut self,
        cycle_actions: &[Option<u8>],
        cycle_source_lines: &[Option<u16>],
        cycle_statuses: &[Option<RobotCycleStatus>],
        cycle_cpu_steps: &mut [Vec<RecordedCpuStep>],
        ground_changes: Vec<GroundAnimationChange>,
    ) {
        let Some(animation) = &mut self.animation else {
            return;
        };
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

    fn record_pending_action_results(&mut self, pending_results: Vec<ActionResult>) {
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

    pub(super) fn should_preserve_program_action_result(&self, robot_index: usize) -> bool {
        self.action_results[robot_index].is_some()
            && matches!(
                &self.action_sources[robot_index],
                ActionSource::Program { runner, .. }
                    if runner.pending_scan_start() || runner.awaits_scan_result()
            )
    }

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
}
