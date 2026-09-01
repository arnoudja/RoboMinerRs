use crate::cpu_step_result::CpuStepResult;
use crate::program_value::as_f64_for_action_arg;
use crate::runner::expression_eval::resume::ExpressionResume;
use crate::runner::expression_eval::schedule::{
    ExpressionWork, ExpressionWorkItem, schedule_expression,
};
use crate::runner::{ExecutableRunner, StepOutcome};
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OngoingExpressionEval {
    pub(crate) work: Vec<ExpressionWorkItem>,
    pub(crate) index: usize,
    pub(crate) values: Vec<CpuStepResult>,
    pub(crate) resume: ExpressionResume,
}

impl OngoingExpressionEval {
    /// Span of the sub-expression this evaluation is about to run, if it has not finished.
    pub(crate) fn current_span(&self) -> Option<SourceSpan> {
        self.work.get(self.index).map(|item| item.span)
    }

    /// Span of the work item most recently advanced past, if any.
    pub(super) fn last_executed_span(&self) -> Option<SourceSpan> {
        self.index
            .checked_sub(1)
            .and_then(|index| self.work.get(index).map(|item| item.span))
    }

    pub(super) fn pop_value(&mut self) -> Option<CpuStepResult> {
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
                eval.values
                    .pop()
                    .map(|value| as_f64_for_action_arg(value.value))
                    .unwrap_or(0.0)
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
}
