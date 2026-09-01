use crate::pending_program_motion::{PendingProgramMotion, ProgramMotionCompletion};
use crate::runner::expression_eval::ExpressionResume;
use crate::runner::{ExecutableRunner, StepOutcome};
use crate::types::*;

impl ExecutableRunner {
    pub(super) fn step_with_result(
        &mut self,
        context: &ExecutionContext,
        action_result: &mut Option<f64>,
    ) -> StepOutcome {
        if self.expression_eval.is_some() {
            return self.step_ongoing_expression(context, action_result);
        }

        if let Some(outcome) = self.handle_continue_program_motion(action_result) {
            return outcome;
        }

        if let Some(outcome) = self.step_frame_boundary() {
            return outcome;
        }

        let Some(frame) = self.stack.last_mut() else {
            return StepOutcome::Done;
        };

        let statement = frame.statements[frame.index].clone();
        self.active_source_line = Some(statement.source_line());
        self.active_source_span = Some(statement.source_span);

        self.step_statement(statement)
    }

    /// Handle end-of-frame: re-check a while/do condition, or pop the finished frame.
    fn step_frame_boundary(&mut self) -> Option<StepOutcome> {
        let repeat_frame = self
            .stack
            .last()
            .filter(|frame| frame.index >= frame.statements.len())
            .map(|frame| (frame.repeat_condition.clone(), frame.repeat_source_span));

        if let Some((Some(condition), repeat_span)) = repeat_frame {
            // Re-attribute to the while/do line before evaluating the condition again
            // (otherwise sticky active_source_line keeps the last body statement).
            if let Some(span) = repeat_span {
                self.set_active_source(Some(span));
            }
            self.start_expression_evaluation(condition, ExpressionResume::RepeatCondition);
            return Some(StepOutcome::Continue);
        }

        if self
            .stack
            .last()
            .is_some_and(|frame| frame.index >= frame.statements.len())
        {
            self.pop_frame();
            return Some(StepOutcome::Continue);
        }

        None
    }

    fn step_statement(&mut self, statement: ExecutableStatement) -> StepOutcome {
        let source_span = statement.source_span;
        match statement.kind {
            ExecutableStatementKind::Action(action) => {
                self.step_action_statement(source_span, action)
            }
            ExecutableStatementKind::DynamicAction(action) => {
                self.step_dynamic_action_statement(action)
            }
            ExecutableStatementKind::Sequence(statements) => {
                self.step_sequence_statement(source_span, statements)
            }
            ExecutableStatementKind::Declare {
                name,
                value_type,
                value,
            } => self.step_declare_statement(source_span, name, value_type, value),
            ExecutableStatementKind::Assign { name, value } => {
                self.step_assign_statement(name, value)
            }
            ExecutableStatementKind::Expression(expression) => {
                self.step_expression_statement(expression)
            }
            ExecutableStatementKind::If {
                condition,
                true_body,
                false_body,
            } => self.step_if_statement(condition, true_body, false_body),
            ExecutableStatementKind::While {
                condition,
                body,
                is_do_while,
            } => self.step_while_statement(source_span, condition, body, is_do_while),
        }
    }

    fn step_action_statement(
        &mut self,
        source_span: SourceSpan,
        action: ExecutableAction,
    ) -> StepOutcome {
        self.last_step_span = Some(source_span);
        if !PendingProgramMotion::is_chunked(action) {
            self.advance_current_statement();
        }
        StepOutcome::Action(action)
    }

    fn advance_current_statement(&mut self) {
        if let Some(frame) = self.stack.last_mut() {
            frame.index += 1;
        }
    }

    fn step_dynamic_action_statement(&mut self, action: ExecutableActionExpression) -> StepOutcome {
        match action {
            ExecutableActionExpression::Move(value) => {
                self.start_expression_evaluation(value, ExpressionResume::DynamicMove);
            }
            ExecutableActionExpression::Rotate(value) => {
                self.start_expression_evaluation(value, ExpressionResume::DynamicRotate);
            }
            ExecutableActionExpression::Dump(value) => {
                self.start_expression_evaluation(value, ExpressionResume::DynamicDump);
            }
        }
        StepOutcome::Continue
    }

    fn step_sequence_statement(
        &mut self,
        source_span: SourceSpan,
        statements: Vec<ExecutableStatement>,
    ) -> StepOutcome {
        self.advance_current_statement();
        self.last_step_span = Some(source_span);
        self.push_frame(statements, None, None, true);
        StepOutcome::Cpu
    }

    fn step_declare_statement(
        &mut self,
        source_span: SourceSpan,
        name: String,
        value_type: ValueType,
        value: Option<ExecutableExpression>,
    ) -> StepOutcome {
        if let Some(value) = value {
            self.start_expression_evaluation(value, ExpressionResume::Declare { name, value_type });
            StepOutcome::Continue
        } else {
            self.variables.declare_default(name, value_type);
            self.advance_current_statement();
            self.last_step_span = Some(source_span);
            StepOutcome::Cpu
        }
    }

    fn step_assign_statement(&mut self, name: String, value: ExecutableExpression) -> StepOutcome {
        self.start_expression_evaluation(value, ExpressionResume::Assign { name });
        StepOutcome::Continue
    }

    fn step_expression_statement(&mut self, expression: ExecutableExpression) -> StepOutcome {
        self.start_expression_evaluation(expression, ExpressionResume::ExpressionStatement);
        StepOutcome::Continue
    }

    fn step_if_statement(
        &mut self,
        condition: ExecutableExpression,
        true_body: Box<ExecutableStatement>,
        false_body: Option<Box<ExecutableStatement>>,
    ) -> StepOutcome {
        self.start_expression_evaluation(
            condition,
            ExpressionResume::If {
                true_body,
                false_body,
            },
        );
        StepOutcome::Continue
    }

    fn step_while_statement(
        &mut self,
        loop_span: SourceSpan,
        condition: ExecutableExpression,
        body: Option<Box<ExecutableStatement>>,
        is_do_while: bool,
    ) -> StepOutcome {
        if is_do_while {
            self.step_do_while_statement(loop_span, condition, body)
        } else {
            let resume_condition = condition.clone();
            self.start_expression_evaluation(
                condition,
                ExpressionResume::While {
                    condition: resume_condition,
                    body,
                    source_span: loop_span,
                },
            );
            StepOutcome::Continue
        }
    }

    fn step_do_while_statement(
        &mut self,
        loop_span: SourceSpan,
        condition: ExecutableExpression,
        body: Option<Box<ExecutableStatement>>,
    ) -> StepOutcome {
        if let Some(body) = body {
            self.advance_current_statement();
            self.push_statement(*body, Some(condition), Some(loop_span));
            StepOutcome::Cpu
        } else if let Some(action) = condition.first_action() {
            if PendingProgramMotion::is_chunked(action) {
                StepOutcome::Action(
                    self.start_pending_program_motion(action, ProgramMotionCompletion::Statement),
                )
            } else {
                StepOutcome::Action(self.queue_pending_action(action))
            }
        } else {
            self.advance_current_statement();
            StepOutcome::Cpu
        }
    }
}
