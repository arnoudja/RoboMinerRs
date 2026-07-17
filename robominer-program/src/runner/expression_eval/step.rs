use super::super::{ExecutableRunner, StepOutcome};
use super::resume::ExpressionResume;
use super::schedule::{ExpressionWork, Truthy, evaluate_operator, schedule_expression};
use crate::pending_physical_action::{
    ContinuePhysicalAction, PendingPhysicalAction, PhysicalCompletion,
};
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OngoingExpressionEval {
    work: Vec<ExpressionWork>,
    index: usize,
    values: Vec<f64>,
    resume: ExpressionResume,
}

impl OngoingExpressionEval {
    pub(crate) fn pending_scan_read(&self) -> bool {
        self.index < self.work.len()
            && matches!(
                self.work[self.index],
                ExpressionWork::PushOreDistance | ExpressionWork::PushOreType
            )
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
            let (value, resume) = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                (eval.values.pop().unwrap_or(0.0), eval.resume.clone())
            };
            self.expression_eval = None;
            return self.finish_expression(resume, value);
        }

        let work = {
            let eval = self.expression_eval.as_ref().expect("expression eval");
            eval.work[eval.index].clone()
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
                    eval.values.push(value);
                    eval.index += 1;
                    return self.complete_expression_work_if_done();
                }
                *action_result = None;
                return StepOutcome::Action(ExecutableAction::StartScan(direction));
            }

            let direction = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                eval.values.pop().unwrap_or(0.0)
            };
            *action_result = None;
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
                eval.values.push(value);
                eval.index += 1;
                return self.complete_expression_work_if_done();
            }

            if !context.scan_complete {
                *action_result = None;
                return StepOutcome::Action(ExecutableAction::AwaitScanResult);
            }

            let value = if matches!(work, ExpressionWork::PushOreDistance) {
                context.scan_distance
            } else {
                context.scan_ore_type
            };
            let eval = self.expression_eval.as_mut().expect("expression eval");
            eval.values.push(value);
            eval.index += 1;
            return self.complete_expression_work_if_done();
        }

        let eval = self.expression_eval.as_mut().expect("expression eval");
        match work {
            ExpressionWork::PushNumber(value) => {
                eval.values.push(value);
                eval.index += 1;
            }
            ExpressionWork::PushVariable(name) => {
                eval.values.push(self.variables.get(&name));
                eval.index += 1;
            }
            ExpressionWork::PushVariableUpdate { name, operator } => {
                let value = match operator {
                    VariableOperator::PreIncrement => self.variables.update(&name, 1.0, true),
                    VariableOperator::PreDecrement => self.variables.update(&name, -1.0, true),
                    VariableOperator::PostIncrement => self.variables.update(&name, 1.0, false),
                    VariableOperator::PostDecrement => self.variables.update(&name, -1.0, false),
                    VariableOperator::None => self.variables.get(&name),
                };
                eval.values.push(value);
                eval.index += 1;
            }
            ExpressionWork::PushTime => {
                eval.values.push(context.time_left as f64);
                eval.index += 1;
            }
            ExpressionWork::PushRobotProperty(property) => {
                eval.values.push(property.value(&context.robot));
                eval.index += 1;
            }
            ExpressionWork::PushOre => {
                let ore_type = eval.values.pop().unwrap_or(0.0) as i32;
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
                eval.values.push(amount);
                eval.index += 1;
            }
            ExpressionWork::PushAction(_) => {
                let value = action_result.take().expect("action result for PushAction");
                eval.values.push(value);
                eval.index += 1;
            }
            ExpressionWork::ApplyUnaryNot => {
                let value = eval.values.pop().unwrap_or(0.0);
                eval.values.push(if value.is_truthy() { 0.0 } else { 1.0 });
                eval.index += 1;
            }
            ExpressionWork::ApplyBinary(operator) => {
                let right = eval.values.pop().unwrap_or(0.0);
                let left = eval.values.pop().unwrap_or(0.0);
                eval.values.push(evaluate_operator(operator, left, right));
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
        if let Some(outcome) = self.handle_continue_physical(action_result) {
            return outcome;
        }

        let action = fixed_action.unwrap_or_else(|| {
            let eval = self.expression_eval.as_mut().expect("expression eval");
            let arg = eval.values.pop().unwrap_or(0.0);
            match eval.work[eval.index] {
                ExpressionWork::PushDynamicMove => ExecutableAction::Move(arg),
                ExpressionWork::PushDynamicRotate => ExecutableAction::Rotate(arg),
                _ => unreachable!("dynamic move/rotate requires matching work item"),
            }
        });

        if !PendingPhysicalAction::is_chunked(action) {
            // move(0) / rotate(0) travel nothing; complete with 0 without awaiting the sim.
            let eval = self.expression_eval.as_mut().expect("expression eval");
            eval.values.push(0.0);
            eval.index += 1;
            return self.complete_expression_work_if_done();
        }

        *action_result = None;
        StepOutcome::Action(self.start_pending_physical(action, PhysicalCompletion::Expression))
    }

    fn step_expression_dump(&mut self, action_result: &mut Option<f64>) -> StepOutcome {
        if let Some(pending) = self.pending_action {
            if let Some(value) = action_result.take() {
                self.pending_action = None;
                let eval = self.expression_eval.as_mut().expect("expression eval");
                eval.values.push(value);
                eval.index += 1;
                return self.complete_expression_work_if_done();
            }
            *action_result = None;
            return StepOutcome::Action(pending);
        }

        let arg = {
            let eval = self.expression_eval.as_mut().expect("expression eval");
            eval.values.pop().unwrap_or(0.0)
        };
        *action_result = None;
        StepOutcome::Action(self.queue_pending_action(ExecutableAction::Dump(arg as i32)))
    }

    pub(crate) fn handle_continue_physical(
        &mut self,
        action_result: &mut Option<f64>,
    ) -> Option<StepOutcome> {
        match PendingPhysicalAction::continue_action(&mut self.pending_physical, action_result) {
            ContinuePhysicalAction::NotActive => None,
            ContinuePhysicalAction::Reemit => {
                *action_result = None;
                Some(StepOutcome::Action(
                    self.pending_physical
                        .as_ref()
                        .expect("reemit requires pending physical action")
                        .action,
                ))
            }
            ContinuePhysicalAction::StatementComplete => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("chunked action requires an active frame");
                frame.index += 1;
                Some(StepOutcome::Cpu)
            }
            ContinuePhysicalAction::ExpressionComplete(value) => {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                eval.values.push(value);
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
            let (value, resume) = {
                let eval = self.expression_eval.as_mut().expect("expression eval");
                (eval.values.pop().unwrap_or(0.0), eval.resume.clone())
            };
            self.expression_eval = None;
            match self.finish_expression(resume, value) {
                StepOutcome::Continue => StepOutcome::Cpu,
                other => other,
            }
        } else {
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
