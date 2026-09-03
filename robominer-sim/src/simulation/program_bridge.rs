//! Runner↔simulation bridge: execution context, CPU loop, scan coordination, and action results.
//!
//! Implements the simulation side of [`robominer_program::pending_action_protocol`].

use robominer_program::{
    CpuStepResult, ExecutableAction, ExecutableRunner, ExecutionContext, ProgramStep, SourceSpan,
};
use std::collections::BTreeMap;

use crate::action_mapping::{
    map_awaiting_executable, robot_action_from_executable, status_for_wait_from_executable,
};
use crate::animation::{RecordedCpuStep, RobotCycleStatus};
use crate::ground::{ScanResult, ScanState};
use crate::physics::ActionResult;
use crate::robot::{ActionSource, ROBOT_ACTION_TYPE_SCAN, RobotAction};

use super::Simulation;

fn push_recorded_cpu_step(
    cpu_steps: &mut Vec<RecordedCpuStep>,
    span: Option<SourceSpan>,
    result: Option<CpuStepResult>,
    variables: BTreeMap<String, CpuStepResult>,
    fallback_line: Option<u16>,
) {
    if let Some(step) = span.and_then(RecordedCpuStep::from_span) {
        cpu_steps.push(step.with_result(result).with_variables(variables));
        return;
    }
    // Unknown/GP spans still burn CPU — record a line-only placeholder when we know
    // the current statement line so the timeline stays visible.
    if let Some(line) = fallback_line.filter(|&line| line != 0) {
        cpu_steps.push(RecordedCpuStep {
            line,
            start_col: 0,
            end_col: 0,
            result,
            variables,
        });
        return;
    }
    debug_assert!(
        span.is_none_or(|span| !span.is_known()),
        "dropped CPU step with unknown span and no current source line"
    );
}

impl Simulation {
    fn perform_scan(
        &mut self,
        robot_index: usize,
        origin: crate::position::Position,
        direction: f64,
    ) -> ScanResult {
        let spec = &self.robots[robot_index].spec;
        if spec.scan_time <= 0 || spec.scan_distance <= 0 {
            ScanResult::empty()
        } else {
            self.ground
                .scan_ore(origin, direction, spec.scan_distance, &self.ore_ids)
        }
    }

    fn start_scan(&mut self, robot_index: usize, direction: f64) -> i32 {
        let scan_time = self.robots[robot_index].spec.scan_time.max(0);
        if scan_time <= 0 || self.robots[robot_index].spec.scan_distance <= 0 {
            self.robots[robot_index].scan_state = ScanState::Complete(ScanResult::empty());
            return 0;
        }

        let origin = self.robots[robot_index].center_position();
        self.robots[robot_index].scan_state = ScanState::Scanning {
            direction,
            origin,
            cycles_remaining: scan_time,
        };
        scan_time
    }

    fn tick_scan(&mut self, robot_index: usize) {
        let ScanState::Scanning {
            direction,
            origin,
            cycles_remaining,
        } = self.robots[robot_index].scan_state.clone()
        else {
            return;
        };

        if cycles_remaining <= 1 {
            let result = self.perform_scan(robot_index, origin, direction);
            self.robots[robot_index].scan_state = ScanState::Complete(result);
        } else {
            self.robots[robot_index].scan_state = ScanState::Scanning {
                direction,
                origin,
                cycles_remaining: cycles_remaining - 1,
            };
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
        context.depot = *robot.depot();
        context.depot_capacity = *robot.depot_capacity();
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
            max_turns: robot.spec.max_turns as f64,
            mining_speed: robot.spec.mining_speed as f64,
            cpu_speed: robot.spec.cpu_speed as f64,
            orientation,
            x_pos,
            y_pos,
        };
        // Opposite-corner spawn-local coords: max - min = size - robot_size on each axis.
        let opposite_span_x = self.ground.size_x() as f64 - robot.spec.robot_size;
        let opposite_span_y = self.ground.size_y() as f64 - robot.spec.robot_size;
        context.area = robominer_program::AreaProperties {
            size_x: opposite_span_x,
            size_y: opposite_span_y,
            container_tax: self.area.container_tax,
            depot_tax: self.area.depot_tax,
            starting_ore_a: self.area.starting_ore[0],
            starting_ore_b: self.area.starting_ore[1],
            starting_ore_c: self.area.starting_ore[2],
            mining_turns: self.area.mining_turns,
            ore_target: self.area.ore_target,
        };
        context
    }

    pub(super) fn run_program_cpu_loop(
        &mut self,
        robot_index: usize,
    ) -> (RobotAction, Option<RobotCycleStatus>, Vec<RecordedCpuStep>) {
        let cpu_speed = self.robots[robot_index].spec.cpu_speed;
        let mut cpu_used = 0;
        let mut cpu_steps = Vec::new();

        loop {
            if cpu_used >= cpu_speed {
                self.action_result_expected[robot_index] = false;
                return (RobotAction::Wait, Some(RobotCycleStatus::Cpu), cpu_steps);
            }

            if !matches!(
                &self.action_sources[robot_index],
                ActionSource::Program { .. }
            ) {
                self.action_result_expected[robot_index] = false;
                return (RobotAction::Wait, Some(RobotCycleStatus::Wait), cpu_steps);
            }

            let span_before = self
                .program_runner(robot_index)
                .and_then(|runner| runner.current_source_span());
            let fallback_line = self
                .program_runner(robot_index)
                .and_then(ExecutableRunner::current_source_line);

            let mut context = self.build_execution_context(robot_index);

            let (step, step_result, step_span, variables) = {
                let ActionSource::Program {
                    program: _, runner, ..
                } = &mut self.action_sources[robot_index]
                else {
                    unreachable!("ActionSource::Program checked above");
                };
                let step = runner.step(&mut context);
                let step_result = runner.take_last_step_result();
                let step_span = runner.take_last_step_span().or(span_before);
                let variables = runner.runtime_variables_snapshot();
                (step, step_result, step_span, variables)
            };
            // Prefer the line after step (active statement may have advanced into the work).
            let fallback_line = self
                .program_runner(robot_index)
                .and_then(ExecutableRunner::current_source_line)
                .or(fallback_line);

            match step {
                ProgramStep::Cpu => {
                    push_recorded_cpu_step(
                        &mut cpu_steps,
                        step_span,
                        step_result,
                        variables,
                        fallback_line,
                    );
                    self.action_results[robot_index] = context.action_result;
                    cpu_used += 1;
                    self.tick_scan(robot_index);
                }
                ProgramStep::Done => {
                    let ActionSource::Program {
                        program, runner, ..
                    } = &mut self.action_sources[robot_index]
                    else {
                        unreachable!("ActionSource::Program checked above");
                    };
                    **runner = program.runner();
                    self.action_results[robot_index] = None;
                    // Restart clears sticky highlight seed so stale lines cannot rematch.
                    self.last_cpu_highlight[robot_index] = None;
                    // Ignore pre-Done recorded steps when reseeding after this CPU loop.
                    self.cpu_highlight_seed_floor[robot_index] = cpu_steps.len();
                    // Empty programs restart immediately; charge budget so we cannot spin forever.
                    cpu_used += 1;
                }
                ProgramStep::Fault => {
                    // Halt without restarting: a corrupted/buggy executable must not livelock.
                    if let ActionSource::Program { runner, .. } =
                        &mut self.action_sources[robot_index]
                    {
                        runner.clear_pending_action_handshake();
                    }
                    self.action_results[robot_index] = None;
                    self.action_result_expected[robot_index] = false;
                    self.pending_sim_motion_chunks[robot_index] = None;
                    return (RobotAction::Wait, Some(RobotCycleStatus::Wait), cpu_steps);
                }
                ProgramStep::Action(ExecutableAction::StartScan(direction)) => {
                    // StartScan returns scan_time synchronously on issue (unlike move/mine).
                    let _ = step_result;
                    let scan_time = self.start_scan(robot_index, direction);
                    let result = CpuStepResult::int_value(i64::from(scan_time));
                    push_recorded_cpu_step(
                        &mut cpu_steps,
                        step_span,
                        Some(result),
                        variables,
                        fallback_line,
                    );
                    self.robots[robot_index].actions_done[ROBOT_ACTION_TYPE_SCAN] += 1;
                    self.action_results[robot_index] = Some(scan_time as f64);
                    self.action_result_expected[robot_index] = false;
                    cpu_used += 1;
                }
                ProgramStep::Action(ExecutableAction::AwaitScanResult) => {
                    // Mid-scan wait: no completed return yet (`r` omitted).
                    push_recorded_cpu_step(
                        &mut cpu_steps,
                        step_span,
                        None,
                        variables,
                        fallback_line,
                    );
                    // Wait out the real scan countdown: one tick per CPU, spanning
                    // turns when remaining work exceeds cpu_speed.
                    self.tick_scan(robot_index);
                    self.action_results[robot_index] = None;
                    self.action_result_expected[robot_index] = false;
                    cpu_used += 1;
                }
                ProgramStep::Action(action) => {
                    // Issuing an awaiting move/rotate/mine/dump has no return yet; omit `r`.
                    debug_assert!(
                        step_result.is_none(),
                        "awaiting Action issue should not produce a step result"
                    );
                    push_recorded_cpu_step(
                        &mut cpu_steps,
                        step_span,
                        None,
                        variables,
                        fallback_line,
                    );
                    let awaits = {
                        let ActionSource::Program { runner, .. } =
                            &self.action_sources[robot_index]
                        else {
                            return (RobotAction::Wait, Some(RobotCycleStatus::Wait), cpu_steps);
                        };
                        runner.awaits_action_result()
                            && robominer_program::await_kind(action).expects_physics_result()
                    };
                    self.action_results[robot_index] = context.action_result;
                    self.action_result_expected[robot_index] = awaits;

                    if awaits {
                        let (pending, robot_action) =
                            map_awaiting_executable(action, self.robots[robot_index].spec());
                        self.pending_sim_motion_chunks[robot_index] = pending;
                        // Pin sticky seed to the issuing Action step (not an earlier micro-step).
                        if let Some(step) = cpu_steps.last() {
                            self.last_cpu_highlight[robot_index] = Some(step.clone());
                        }
                        let status = if matches!(robot_action, RobotAction::Wait) {
                            Some(status_for_wait_from_executable(action))
                        } else {
                            None
                        };
                        return (robot_action, status, cpu_steps);
                    }

                    let robot_action =
                        robot_action_from_executable(action, &self.robots[robot_index].spec);
                    let status = if matches!(robot_action, RobotAction::Wait) {
                        Some(status_for_wait_from_executable(action))
                    } else {
                        None
                    };
                    return (robot_action, status, cpu_steps);
                }
            }
        }
    }

    pub(super) fn record_action_result(&mut self, robot_index: usize, result: ActionResult) {
        if matches!(result, ActionResult::None) {
            // Wait (or other no-ops) while motion is still pending: remaining distance is
            // within epsilon or speed is zero. Finish with the accumulated travel so the
            // runner is not left awaiting a result that will never arrive.
            if let Some(pending) = self.pending_sim_motion_chunks[robot_index].take() {
                self.action_results[robot_index] = Some(pending.accumulated());
            }
            return;
        }

        let value = match result {
            ActionResult::Mine => self.robots[robot_index].last_mined() as f64,
            ActionResult::Value(value) => value,
            ActionResult::Move { .. } | ActionResult::None => return,
        };

        if let Some(pending) = &mut self.pending_sim_motion_chunks[robot_index] {
            if pending.record_step(value, self.robots[robot_index].spec()) {
                self.action_results[robot_index] = Some(pending.accumulated());
                self.pending_sim_motion_chunks[robot_index] = None;
            } else {
                self.action_results[robot_index] = None;
            }
        } else if self.action_result_expected[robot_index] {
            self.action_results[robot_index] = Some(value);
        }
    }
}
