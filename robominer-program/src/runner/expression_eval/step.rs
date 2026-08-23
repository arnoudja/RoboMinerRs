use super::super::{ExecutableRunner, StepOutcome};
use super::resume::ExpressionResume;
use super::schedule::{
    ExpressionWork, ExpressionWorkItem, Truthy, evaluate_operator, schedule_expression,
};
use crate::cpu_step_result::{CpuStepResult, CpuStepResultKind};
use crate::pending_program_motion::{
    ContinueProgramMotion, PendingProgramMotion, ProgramMotionCompletion,
};
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OngoingExpressionEval {
    work: Vec<ExpressionWorkItem>,
    index: usize,
    values: Vec<CpuStepResult>,
    resume: ExpressionResume,
}

impl OngoingExpressionEval {
    /// Span of the sub-expression this evaluation is about to run, if it has not finished.
    pub(crate) fn current_span(&self) -> Option<SourceSpan> {
        self.work.get(self.index).map(|item| item.span)
    }

    /// Span of the work item most recently advanced past, if any.
    fn last_executed_span(&self) -> Option<SourceSpan> {
        self.index
            .checked_sub(1)
            .and_then(|index| self.work.get(index).map(|item| item.span))
    }

    fn pop_value(&mut self) -> Option<CpuStepResult> {
        self.values.pop()
    }
}

impl ExecutableRunner {
    pub(crate) fn start_expression_evaluation(
        &mut self,
        expression: ExecutableExpression,
        resume: ExpressionResume,
    ) {
        let mut work = Vec::new();
        schedule_expression(&mut work, &expression);
        self.expression_eval = Some(OngoingExpressionEval {
            work,
            index: 0,
            values: Vec::new(),
            resume,
        });
    }

    pub(crate) fn step_ongoing_expression(
        &mut self,
        context: &ExecutionContext,
        action_result: &mut Option<f64>,
    ) -> StepOutcome {
        if self
            .expression_eval
            .as_ref()
            .is_some_and(|eval| eval.index >= eval.work.len())
        {
            // Same finish path as mid-eval completion (Continue burns one CPU cycle).
            return self.complete_expression_work_if_done();
        }

        let Some(eval) = self.expression_eval.as_ref() else {
            return self.abort_with_fault();
        };
        let work_span = eval.work.get(eval.index).map(|item| item.span);
        let Some(work_item) = eval.work.get(eval.index) else {
            return self.abort_with_fault();
        };
        let work = work_item.kind.clone();

        if let ExpressionWork::PushAction(action) = &work {
            if matches!(
                action,
                ExecutableAction::Move(_) | ExecutableAction::Rotate(_)
            ) {
                // Zero move/rotate must complete immediately: the sim maps them to Wait
                // and never returns an action result, which would livelock pending state.
                return self.step_expression_move_or_rotate(action_result, Some(*action));
            }
            if action_result.is_none() {
                self.last_step_span = work_span;
                return StepOutcome::Action(self.queue_pending_action(*action));
            }
            self.pending_action = None;
        }

        if matches!(
            work,
            ExpressionWork::PushDynamicMove | ExpressionWork::PushDynamicRotate
        ) {
            return self.step_expression_move_or_rotate(action_result, None);
        }

        if matches!(work, ExpressionWork::PushDynamicDump) {
            return self.step_expression_dump(action_result);
        }

        if let ExpressionWork::PushStartScan = &work {
            if let Some(ExecutableAction::StartScan(direction)) = self.pending_action {
                if let Some(value) = action_result.take() {
                    self.pending_action = None;
                    let Some(eval) = self.expression_eval.as_mut() else {
                        return self.abort_with_fault();
                    };
                    eval.values.push(CpuStepResult::for_action(
                        ExecutableAction::StartScan(0.0),
                        value,
                    ));
                    eval.index += 1;
                    return self.complete_expression_work_if_done();
                }
                *action_result = None;
                self.last_step_span = work_span;
                return StepOutcome::Action(ExecutableAction::StartScan(direction));
            }

            // scan() with no argument defaults to direction 0; only pop when an arg was scheduled.
            let direction = {
                let Some(eval) = self.expression_eval.as_mut() else {
                    return self.abort_with_fault();
                };
                eval.values.pop().map(|value| value.value).unwrap_or(0.0)
            };
            *action_result = None;
            self.last_step_span = work_span;
            return StepOutcome::Action(
                self.queue_pending_action(ExecutableAction::StartScan(direction)),
            );
        }

        if matches!(
            work,
            ExpressionWork::PushOreDistance | ExpressionWork::PushOreType
        ) {
            if !context.scan_started {
                let value = if matches!(work, ExpressionWork::PushOreDistance) {
                    -1.0
                } else {
                    0.0
                };
                let Some(eval) = self.expression_eval.as_mut() else {
                    return self.abort_with_fault();
                };
                eval.values
                    .push(if matches!(work, ExpressionWork::PushOreDistance) {
                        CpuStepResult::for_ore_distance(value)
                    } else {
                        CpuStepResult::for_ore_type(value)
                    });
                eval.index += 1;
                return self.complete_expression_work_if_done();
            }

            if !context.scan_complete {
                *action_result = None;
                self.last_step_span = work_span;
                return StepOutcome::Action(ExecutableAction::AwaitScanResult);
            }

            let value = if matches!(work, ExpressionWork::PushOreDistance) {
                context.scan_distance
            } else {
                context.scan_ore_type
            };
            let Some(eval) = self.expression_eval.as_mut() else {
                return self.abort_with_fault();
            };
            eval.values
                .push(if matches!(work, ExpressionWork::PushOreDistance) {
                    CpuStepResult::for_ore_distance(value)
                } else {
                    CpuStepResult::for_ore_type(value)
                });
            eval.index += 1;
            return self.complete_expression_work_if_done();
        }

        if let Err(()) = self.apply_expression_work(context, action_result, work) {
            return self.abort_with_fault();
        }

        self.complete_expression_work_if_done()
    }

    /// Apply one scheduled expression work item. Returns `Err(())` on invariant failure.
    fn apply_expression_work(
        &mut self,
        context: &ExecutionContext,
        action_result: &mut Option<f64>,
        work: ExpressionWork,
    ) -> Result<(), ()> {
        let eval = self.expression_eval.as_mut().ok_or(())?;
        match work {
            ExpressionWork::PushNumber(value) => {
                eval.values.push(CpuStepResult::for_number_literal(value));
                eval.index += 1;
            }
            ExpressionWork::PushBool(value) => {
                eval.values
                    .push(CpuStepResult::bool_value(if value { 1.0 } else { 0.0 }));
                eval.index += 1;
            }
            ExpressionWork::PushVariable(name) => {
                eval.values.push(self.variables.get_typed(&name));
                eval.index += 1;
            }
            ExpressionWork::PushVariableUpdate { name, operator } => {
                let result = match operator {
                    VariableOperator::PreIncrement => self.variables.update(&name, 1.0, true),
                    VariableOperator::PreDecrement => self.variables.update(&name, -1.0, true),
                    VariableOperator::PostIncrement => self.variables.update(&name, 1.0, false),
                    VariableOperator::PostDecrement => self.variables.update(&name, -1.0, false),
                    VariableOperator::None => self.variables.get_typed(&name),
                };
                eval.values.push(result);
                eval.index += 1;
            }
            ExpressionWork::PushTime => {
                eval.values
                    .push(CpuStepResult::int_value(context.time_left as f64));
                eval.index += 1;
            }
            ExpressionWork::PushRobotProperty(property) => {
                let value = property
                    .stored_ore_value(&context.ore)
                    .or_else(|| property.depot_value(&context.depot, &context.depot_capacity))
                    .or_else(|| property.value(&context.robot))
                    .ok_or(())?;
                eval.values
                    .push(CpuStepResult::for_robot_property(property, value));
                eval.index += 1;
            }
            ExpressionWork::PushAreaProperty(property) => {
                let value = property.value(&context.area);
                eval.values
                    .push(CpuStepResult::for_area_property(property, value));
                eval.index += 1;
            }
            // Deprecated: prefer robot.oreStored / robot.oreStoredA|B|C.
            ExpressionWork::PushOre => {
                let ore_type = eval.values.pop().ok_or(())?.value as i32;
                let amount = if ore_type == 0 {
                    context.ore.iter().sum::<i32>() as f64
                } else if ore_type > 0 {
                    context
                        .ore
                        .get((ore_type - 1) as usize)
                        .copied()
                        .unwrap_or(0) as f64
                } else {
                    0.0
                };
                eval.values.push(CpuStepResult::int_value(amount));
                eval.index += 1;
            }
            ExpressionWork::PushAction(action) => {
                let value = action_result.take().ok_or(())?;
                eval.values.push(CpuStepResult::for_action(action, value));
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryNot => {
                let value = eval.values.pop().ok_or(())?.value;
                eval.values
                    .push(CpuStepResult::bool_value(if value.is_truthy() {
                        0.0
                    } else {
                        1.0
                    }));
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryMinus => {
                let operand = eval.values.pop().ok_or(())?;
                let value = -operand.value;
                eval.values
                    .push(if operand.kind == CpuStepResultKind::Float {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    });
                eval.index += 1;
            }
            ExpressionWork::ApplyAbs => {
                let operand = eval.values.pop().ok_or(())?;
                let value = operand.value.abs();
                eval.values
                    .push(if operand.kind == CpuStepResultKind::Float {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    });
                eval.index += 1;
            }
            ExpressionWork::ApplySqrt => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.sqrt()));
                eval.index += 1;
            }
            ExpressionWork::ApplySin => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().sin()));
                eval.index += 1;
            }
            ExpressionWork::ApplyCos => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().cos()));
                eval.index += 1;
            }
            ExpressionWork::ApplyTan => {
                let operand = eval.values.pop().ok_or(())?;
                eval.values
                    .push(CpuStepResult::float_value(operand.value.to_radians().tan()));
                eval.index += 1;
            }
            ExpressionWork::ApplyMin => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = left.value.min(right.value);
                eval.values.push(
                    if matches!(left.kind, CpuStepResultKind::Float)
                        || matches!(right.kind, CpuStepResultKind::Float)
                    {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    },
                );
                eval.index += 1;
            }
            ExpressionWork::ApplyMax => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = left.value.max(right.value);
                eval.values.push(
                    if matches!(left.kind, CpuStepResultKind::Float)
                        || matches!(right.kind, CpuStepResultKind::Float)
                    {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
                    },
                );
                eval.index += 1;
            }
            ExpressionWork::ApplyBinary(operator) => {
                let right = eval.values.pop().ok_or(())?;
                let left = eval.values.pop().ok_or(())?;
                let value = evaluate_operator(operator, left.value, right.value);
                eval.values.push(CpuStepResult::for_binary_operator(
                    operator, left.kind, right.kind, value,
                ));
                eval.index += 1;
            }
            ExpressionWork::PushStartScan
            | ExpressionWork::PushDynamicMove
            | ExpressionWork::PushDynamicRotate
            | ExpressionWork::PushDynamicDump
            | ExpressionWork::PushOreDistance
            | ExpressionWork::PushOreType => return Err(()),
        }
        Ok(())
    }

    fn step_expression_move_or_rotate(
        &mut self,
        action_result: &mut Option<f64>,
        fixed_action: Option<ExecutableAction>,
    ) -> StepOutcome {
        if let Some(outcome) = self.handle_continue_program_motion(action_result) {
            return outcome;
        }

        let action = if let Some(action) = fixed_action {
            action
        } else {
            let Some(eval) = self.expression_eval.as_mut() else {
                return self.abort_with_fault();
            };
            let Some(arg) = eval.values.pop() else {
                return self.abort_with_fault();
            };
            match eval.work.get(eval.index).map(|item| &item.kind) {
                Some(ExpressionWork::PushDynamicMove) => ExecutableAction::Move(arg.value),
                Some(ExpressionWork::PushDynamicRotate) => ExecutableAction::Rotate(arg.value),
                _ => return self.abort_with_fault(),
            }
        };

        if !PendingProgramMotion::is_chunked(action) {
            // move(0) / rotate(0) travel nothing; complete with 0 without awaiting the sim.
            let Some(eval) = self.expression_eval.as_mut() else {
                return self.abort_with_fault();
            };
            eval.values.push(CpuStepResult::for_action(action, 0.0));
            eval.index += 1;
            return self.complete_expression_work_if_done();
        }

        *action_result = None;
        self.last_step_span = self
            .expression_eval
            .as_ref()
            .and_then(OngoingExpressionEval::current_span);
        StepOutcome::Action(
            self.start_pending_program_motion(action, ProgramMotionCompletion::Expression),
        )
    }

    fn step_expression_dump(&mut self, action_result: &mut Option<f64>) -> StepOutcome {
        if let Some(pending) = self.pending_action {
            if let Some(value) = action_result.take() {
                self.pending_action = None;
                let Some(eval) = self.expression_eval.as_mut() else {
                    return self.abort_with_fault();
                };
                eval.values.push(CpuStepResult::for_action(pending, value));
                eval.index += 1;
                return self.complete_expression_work_if_done();
            }
            *action_result = None;
            self.last_step_span = self
                .expression_eval
                .as_ref()
                .and_then(OngoingExpressionEval::current_span);
            return StepOutcome::Action(pending);
        }

        let arg = {
            let Some(eval) = self.expression_eval.as_mut() else {
                return self.abort_with_fault();
            };
            let Some(value) = eval.values.pop() else {
                return self.abort_with_fault();
            };
            value.value
        };
        *action_result = None;
        self.last_step_span = self
            .expression_eval
            .as_ref()
            .and_then(OngoingExpressionEval::current_span);
        StepOutcome::Action(self.queue_pending_action(ExecutableAction::Dump(arg as i32)))
    }

    pub(crate) fn handle_continue_program_motion(
        &mut self,
        action_result: &mut Option<f64>,
    ) -> Option<StepOutcome> {
        let pending_action = self.pending_program_motion.map(|pending| pending.action);
        match PendingProgramMotion::continue_action(&mut self.pending_program_motion, action_result)
        {
            ContinueProgramMotion::NotActive => None,
            ContinueProgramMotion::Reemit => {
                *action_result = None;
                self.last_step_span = pending_action.and_then(|action| {
                    // Fall back to active statement span for re-emitted motion chunks.
                    let _ = action;
                    self.active_source_span
                });
                let Some(pending) = self.pending_program_motion.as_ref() else {
                    return Some(self.abort_with_fault());
                };
                Some(StepOutcome::Action(pending.action))
            }
            ContinueProgramMotion::StatementComplete(value) => {
                let Some(frame) = self.stack.last_mut() else {
                    return Some(self.abort_with_fault());
                };
                frame.index += 1;
                let action = pending_action.unwrap_or(ExecutableAction::Move(0.0));
                self.last_step_result = Some(CpuStepResult::for_action(action, value));
                self.last_step_span = self.active_source_span;
                Some(StepOutcome::Cpu)
            }
            ContinueProgramMotion::ExpressionComplete(value) => {
                let action = pending_action.unwrap_or(ExecutableAction::Move(0.0));
                let Some(eval) = self.expression_eval.as_mut() else {
                    return Some(self.abort_with_fault());
                };
                eval.values.push(CpuStepResult::for_action(action, value));
                eval.index += 1;
                Some(self.complete_expression_work_if_done())
            }
        }
    }

    pub(super) fn complete_expression_work_if_done(&mut self) -> StepOutcome {
        if self
            .expression_eval
            .as_ref()
            .is_some_and(|eval| eval.index >= eval.work.len())
        {
            let finished = {
                let Some(eval) = self.expression_eval.as_mut() else {
                    return self.abort_with_fault();
                };
                let span = eval
                    .last_executed_span()
                    .or_else(|| eval.work.last().map(|item| item.span));
                match eval.pop_value() {
                    Some(result) => Some((result, eval.resume.clone(), span)),
                    None => None,
                }
            };
            let Some((result, resume, span)) = finished else {
                return self.abort_with_fault();
            };
            self.expression_eval = None;
            self.last_step_result = Some(result);
            self.last_step_span = span;
            let dynamic_call = matches!(
                resume,
                ExpressionResume::DynamicMove
                    | ExpressionResume::DynamicRotate
                    | ExpressionResume::DynamicDump
            );
            match self.finish_expression(resume, result.value) {
                StepOutcome::Continue => StepOutcome::Cpu,
                StepOutcome::Action(action) => {
                    // Issuing an action has no return yet; the expression value was the
                    // argument (e.g. move distance), not a completed action result.
                    self.last_step_result = None;
                    // Dynamic move/rotate/dump: highlight the call/statement, not the arg token.
                    if dynamic_call {
                        self.last_step_span = self.active_source_span;
                    }
                    StepOutcome::Action(action)
                }
                other => other,
            }
        } else {
            self.last_step_result = self
                .expression_eval
                .as_ref()
                .and_then(|eval| eval.values.last().copied());
            self.last_step_span = self
                .expression_eval
                .as_ref()
                .and_then(OngoingExpressionEval::last_executed_span);
            StepOutcome::Cpu
        }
    }

    pub(super) fn finish_expression(
        &mut self,
        resume: ExpressionResume,
        value: f64,
    ) -> StepOutcome {
        match self.apply_expression_resume(resume, value) {
            super::resume::ExpressionComplete::Continue => StepOutcome::Continue,
            super::resume::ExpressionComplete::Step(step) => match step {
                ProgramStep::Action(action) => StepOutcome::Action(action),
                ProgramStep::Done => StepOutcome::Done,
                ProgramStep::Cpu => StepOutcome::Cpu,
                ProgramStep::Fault => StepOutcome::Fault,
            },
            super::resume::ExpressionComplete::Fault => StepOutcome::Fault,
        }
    }
}
