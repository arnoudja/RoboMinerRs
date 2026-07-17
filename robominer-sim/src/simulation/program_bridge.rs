//! Runner↔simulation bridge: execution context, CPU loop, scan coordination, and action results.
//!
//! Implements the simulation side of [`robominer_program::pending_action_protocol`].

use robominer_program::{ExecutableAction, ExecutionContext, ProgramStep};

use crate::action_mapping::{map_awaiting_executable, robot_action_from_executable};
use crate::ground::{ScanResult, ScanState};
use crate::physics::ActionResult;
use crate::robot::{ActionSource, ROBOT_ACTION_TYPE_SCAN, RobotAction};

use super::Simulation;

impl Simulation {
    fn perform_scan(&mut self, robot_index: usize, direction: f64) -> ScanResult {
        let robot = &self.robots[robot_index];
        let spec = &robot.spec;
        if spec.scan_time <= 0 || spec.scan_distance <= 0 {
            ScanResult::empty()
        } else {
            self.ground.scan_ore(
                robot.center_position(),
                direction,
                spec.scan_distance,
                &self.ore_ids,
            )
        }
    }

    fn start_scan(&mut self, robot_index: usize, direction: f64) -> i32 {
        let scan_time = self.robots[robot_index].spec.scan_time.max(0);
        if scan_time <= 0 || self.robots[robot_index].spec.scan_distance <= 0 {
            self.robots[robot_index].scan_state = ScanState::Complete(ScanResult::empty());
            return 0;
        }

        self.robots[robot_index].scan_state = ScanState::Scanning {
            direction,
            cycles_remaining: scan_time,
        };
        scan_time
    }

    fn tick_scan(&mut self, robot_index: usize) {
        let ScanState::Scanning {
            direction,
            cycles_remaining,
        } = self.robots[robot_index].scan_state.clone()
        else {
            return;
        };

        if cycles_remaining <= 1 {
            let result = self.perform_scan(robot_index, direction);
            self.robots[robot_index].scan_state = ScanState::Complete(result);
        } else {
            self.robots[robot_index].scan_state = ScanState::Scanning {
                direction,
                cycles_remaining: cycles_remaining - 1,
            };
        }
    }

    fn complete_scan_now(&mut self, robot_index: usize) -> i32 {
        match &self.robots[robot_index].scan_state {
            ScanState::Scanning {
                direction,
                cycles_remaining,
            } => {
                let remaining = *cycles_remaining;
                let result = self.perform_scan(robot_index, *direction);
                self.robots[robot_index].scan_state = ScanState::Complete(result);
                remaining
            }
            _ => 0,
        }
    }

    fn build_execution_context(&self, robot_index: usize) -> ExecutionContext {
        let robot = &self.robots[robot_index];
        let snapshot = robot.scan_snapshot();
        let mut context = ExecutionContext::from_runtime(
            robot.time_left(),
            *robot.ore(),
            self.action_results[robot_index],
        );
        context.scan_time = robot.spec.scan_time;
        context.scan_started = snapshot.started;
        context.scan_complete = snapshot.complete;
        context.scan_distance = snapshot.distance;
        context.scan_ore_type = snapshot.ore_type;
        let center = robot.effective_center_position();
        let (x_pos, y_pos, orientation) = robominer_program::rally_robot_pose(
            center.x,
            center.y,
            center.orientation,
            robot.initial_center_x,
            robot.initial_center_y,
            robot.initial_orientation,
        );
        context.robot = robominer_program::RobotProperties {
            forward_speed: robot.spec.forward_speed,
            backward_speed: robot.spec.backward_speed,
            rotate_speed: robot.spec.rotate_speed as f64,
            scan_time: robot.spec.scan_time as f64,
            scan_distance: robot.spec.scan_distance as f64,
            ore_cap: robot.spec.max_ore as f64,
            max_cycles: robot.spec.max_turns as f64,
            mining_speed: robot.spec.mining_speed as f64,
            cpu_speed: robot.spec.cpu_speed as f64,
            orientation,
            x_pos,
            y_pos,
        };
        context
    }

    pub(super) fn run_program_cpu_loop(&mut self, robot_index: usize) -> RobotAction {
        let cpu_speed = self.robots[robot_index].spec.cpu_speed;
        let mut cpu_used = 0;

        loop {
            let extend_budget = {
                let ActionSource::Program { runner, .. } = &self.action_sources[robot_index] else {
                    self.action_result_expected[robot_index] = false;
                    return RobotAction::Wait;
                };
                runner.awaits_scan_result() || runner.has_pending_scan_completion()
            };

            if cpu_used >= cpu_speed && !extend_budget {
                self.action_result_expected[robot_index] = false;
                return RobotAction::Wait;
            }

            let mut context = self.build_execution_context(robot_index);

            let step = {
                let ActionSource::Program { program: _, runner } =
                    &mut self.action_sources[robot_index]
                else {
                    self.action_result_expected[robot_index] = false;
                    return RobotAction::Wait;
                };
                runner.step(&mut context)
            };

            match step {
                ProgramStep::Cpu => {
                    self.action_results[robot_index] = context.action_result;
                    cpu_used += 1;
                    self.tick_scan(robot_index);
                }
                ProgramStep::Done => {
                    let ActionSource::Program { program, runner } =
                        &mut self.action_sources[robot_index]
                    else {
                        return RobotAction::Wait;
                    };
                    **runner = program.runner();
                    self.action_results[robot_index] = None;
                }
                ProgramStep::Action(ExecutableAction::StartScan(direction)) => {
                    let scan_time = self.start_scan(robot_index, direction);
                    self.robots[robot_index].actions_done[ROBOT_ACTION_TYPE_SCAN] += 1;
                    self.action_results[robot_index] = Some(scan_time as f64);
                    self.action_result_expected[robot_index] = false;
                    cpu_used += 1;
                }
                ProgramStep::Action(ExecutableAction::AwaitScanResult) => {
                    let remaining = self.complete_scan_now(robot_index);
                    self.action_results[robot_index] = None;
                    self.action_result_expected[robot_index] = false;
                    cpu_used += remaining.max(1);
                }
                ProgramStep::Action(action) => {
                    let awaits = {
                        let ActionSource::Program { runner, .. } =
                            &self.action_sources[robot_index]
                        else {
                            return RobotAction::Wait;
                        };
                        runner.awaits_action_result()
                            && robominer_program::await_kind(action).expects_physics_result()
                    };
                    self.action_results[robot_index] = context.action_result;
                    self.action_result_expected[robot_index] = awaits;

                    if awaits {
                        let (pending, robot_action) =
                            map_awaiting_executable(action, self.robots[robot_index].spec());
                        self.pending_expression_actions[robot_index] = pending;
                        return robot_action;
                    }

                    return robot_action_from_executable(action, &self.robots[robot_index].spec);
                }
            }
        }
    }

    pub(super) fn record_action_result(&mut self, robot_index: usize, result: ActionResult) {
        if matches!(result, ActionResult::None) {
            // Wait (or other no-ops) while motion is still pending: remaining distance is
            // within epsilon or speed is zero. Finish with the accumulated travel so the
            // runner is not left awaiting a result that will never arrive.
            if let Some(pending) = self.pending_expression_actions[robot_index].take() {
                self.action_results[robot_index] = Some(pending.accumulated());
            }
            return;
        }

        let value = match result {
            ActionResult::Mine => self.robots[robot_index].last_mined() as f64,
            ActionResult::Value(value) => value,
            ActionResult::Move { .. } | ActionResult::None => return,
        };

        if let Some(pending) = &mut self.pending_expression_actions[robot_index] {
            if pending.record_step(value, self.robots[robot_index].spec()) {
                self.action_results[robot_index] = Some(pending.accumulated());
                self.pending_expression_actions[robot_index] = None;
            } else {
                self.action_results[robot_index] = None;
            }
        } else if self.action_result_expected[robot_index] {
            self.action_results[robot_index] = Some(value);
        }
    }
}
