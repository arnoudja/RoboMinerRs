use super::OngoingExpressionEval;
use crate::cpu_step_result::CpuStepResult;
use crate::pending_program_motion::{
    ContinueProgramMotion, PendingProgramMotion, ProgramMotionCompletion,
};
use crate::runner::expression_eval::schedule::ExpressionWork;
use crate::runner::{ExecutableRunner, StepOutcome};
use crate::types::*;

impl ExecutableRunner {
    pub(super) fn step_expression_move_or_rotate(
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
                Some(ExpressionWork::PushDynamicMove) => ExecutableAction::Move(arg.as_f64()),
                Some(ExpressionWork::PushDynamicRotate) => ExecutableAction::Rotate(arg.as_f64()),
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

    pub(super) fn step_expression_dump(&mut self, action_result: &mut Option<f64>) -> StepOutcome {
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
            value.as_f64()
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
}
