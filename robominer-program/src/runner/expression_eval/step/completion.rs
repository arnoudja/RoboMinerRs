use super::OngoingExpressionEval;
use crate::runner::expression_eval::resume::{ExpressionComplete, ExpressionResume};
use crate::runner::{ExecutableRunner, StepOutcome};
use crate::types::*;

impl ExecutableRunner {
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
            ExpressionComplete::Continue => StepOutcome::Continue,
            ExpressionComplete::Step(step) => match step {
                ProgramStep::Action(action) => StepOutcome::Action(action),
                ProgramStep::Done => StepOutcome::Done,
                ProgramStep::Cpu => StepOutcome::Cpu,
                ProgramStep::Fault => StepOutcome::Fault,
            },
            ExpressionComplete::Fault => StepOutcome::Fault,
        }
    }
}
