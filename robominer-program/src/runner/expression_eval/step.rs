use super::super::{ExecutableRunner, StepOutcome};
use super::resume::ExpressionResume;
use super::schedule::{
    ExpressionWork, ExpressionWorkItem, Truthy, evaluate_operator, schedule_expression,
};
use crate::cpu_step_result::CpuStepResult;
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
            let (result, resume, span) = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                let span = eval.work.last().map(|item| item.span);
                (
                    eval.values
                        .pop()
                        .unwrap_or_else(|| CpuStepResult::int_value(0.0)),
                    eval.resume.clone(),
                    span,
                )
            };
            self.expression_eval = None;
            self.last_step_result = Some(result);
            self.last_step_span = span;
            return self.finish_expression(resume, result.value);
        }

        let work_span = {
            let eval = self.expression_eval.as_ref().expect("expression eval");
            eval.work.get(eval.index).map(|item| item.span)
        };
        let work = {
            let eval = self.expression_eval.as_ref().expect("expression eval");
            eval.work[eval.index].kind.clone()
        };

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
                    let eval = self.expression_eval.as_mut().expect("expression eval");
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

            let direction = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                eval.values
                    .pop()
                    .unwrap_or_else(|| CpuStepResult::int_value(0.0))
                    .value
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
                let eval = self.expression_eval.as_mut().expect("expression eval");
                eval.values
                    .push(if matches!(work, ExpressionWork::PushOreDistance) {
                        CpuStepResult::float_value(value)
                    } else {
                        CpuStepResult::int_value(value)
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
            let eval = self.expression_eval.as_mut().expect("expression eval");
            eval.values
                .push(if matches!(work, ExpressionWork::PushOreDistance) {
                    CpuStepResult::float_value(value)
                } else {
                    CpuStepResult::int_value(value)
                });
            eval.index += 1;
            return self.complete_expression_work_if_done();
        }

        let eval = self.expression_eval.as_mut().expect("expression eval");
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
                    .or_else(|| property.value(&context.robot))
                    .expect("robot property should resolve");
                eval.values
                    .push(CpuStepResult::for_robot_property(property, value));
                eval.index += 1;
            }
            // Deprecated: prefer robot.oreStored / robot.oreStoredA|B|C.
            ExpressionWork::PushOre => {
                let ore_type = eval
                    .values
                    .pop()
                    .unwrap_or_else(|| CpuStepResult::int_value(0.0))
                    .value as i32;
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
                let value = action_result.take().expect("action result for PushAction");
                eval.values.push(CpuStepResult::for_action(action, value));
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryNot => {
                let value = eval
                    .values
                    .pop()
                    .unwrap_or_else(|| CpuStepResult::int_value(0.0))
                    .value;
                eval.values
                    .push(CpuStepResult::bool_value(if value.is_truthy() {
                        0.0
                    } else {
                        1.0
                    }));
                eval.index += 1;
            }
            ExpressionWork::ApplyBinary(operator) => {
                let right = eval
                    .values
                    .pop()
                    .unwrap_or_else(|| CpuStepResult::int_value(0.0));
                let left = eval
                    .values
                    .pop()
                    .unwrap_or_else(|| CpuStepResult::int_value(0.0));
                let value = evaluate_operator(operator, left.value, right.value);
                eval.values.push(CpuStepResult::for_binary_operator(
                    operator, left.kind, right.kind, value,
                ));
                eval.index += 1;
            }
            ExpressionWork::PushStartScan => unreachable!("PushStartScan handled above"),
            ExpressionWork::PushDynamicMove
            | ExpressionWork::PushDynamicRotate
            | ExpressionWork::PushDynamicDump => unreachable!("dynamic actions handled above"),
            ExpressionWork::PushOreDistance | ExpressionWork::PushOreType => {
                unreachable!("ore distance/type handled above")
            }
        }

        self.complete_expression_work_if_done()
    }

    fn step_expression_move_or_rotate(
        &mut self,
        action_result: &mut Option<f64>,
        fixed_action: Option<ExecutableAction>,
    ) -> StepOutcome {
        if let Some(outcome) = self.handle_continue_program_motion(action_result) {
            return outcome;
        }

        let action = fixed_action.unwrap_or_else(|| {
            let eval = self.expression_eval.as_mut().expect("expression eval");
            let arg = eval
                .values
                .pop()
                .unwrap_or_else(|| CpuStepResult::int_value(0.0))
                .value;
            match eval.work[eval.index].kind {
                ExpressionWork::PushDynamicMove => ExecutableAction::Move(arg),
                ExpressionWork::PushDynamicRotate => ExecutableAction::Rotate(arg),
                _ => unreachable!("dynamic move/rotate requires matching work item"),
            }
        });

        if !PendingProgramMotion::is_chunked(action) {
            // move(0) / rotate(0) travel nothing; complete with 0 without awaiting the sim.
            let eval = self.expression_eval.as_mut().expect("expression eval");
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
                let eval = self.expression_eval.as_mut().expect("expression eval");
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
            let eval = self.expression_eval.as_mut().expect("expression eval");
            eval.values
                .pop()
                .unwrap_or_else(|| CpuStepResult::int_value(0.0))
                .value
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
                Some(StepOutcome::Action(
                    self.pending_program_motion
                        .as_ref()
                        .expect("reemit requires pending_program_motion")
                        .action,
                ))
            }
            ContinueProgramMotion::StatementComplete => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("chunked action requires an active frame");
                frame.index += 1;
                self.last_step_span = self.active_source_span;
                Some(StepOutcome::Cpu)
            }
            ContinueProgramMotion::ExpressionComplete(value) => {
                let action = pending_action.unwrap_or(ExecutableAction::Move(0.0));
                let eval = self.expression_eval.as_mut().expect("expression eval");
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
            let (result, resume, span) = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                let span = eval
                    .last_executed_span()
                    .or_else(|| eval.work.last().map(|item| item.span));
                (
                    eval.values
                        .pop()
                        .unwrap_or_else(|| CpuStepResult::int_value(0.0)),
                    eval.resume.clone(),
                    span,
                )
            };
            self.expression_eval = None;
            self.last_step_result = Some(result);
            self.last_step_span = span;
            match self.finish_expression(resume, result.value) {
                StepOutcome::Continue => StepOutcome::Cpu,
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
            },
        }
    }
}
