use super::super::{ExecutableRunner, StepOutcome};
use crate::cpu_step_result::CpuStepResult;
use crate::pending_program_motion::PendingProgramMotion;
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionResume {
    RepeatCondition,
    If {
        true_body: Box<ExecutableStatement>,
        false_body: Option<Box<ExecutableStatement>>,
    },
    While {
        condition: ExecutableExpression,
        body: Option<Box<ExecutableStatement>>,
        source_span: SourceSpan,
    },
    Declare {
        name: String,
        value_type: ValueType,
    },
    Assign {
        name: String,
    },
    ExpressionStatement,
    DynamicMove,
    DynamicRotate,
    DynamicDump,
    /// Finish a `return <expr>;` inside a user function.
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionComplete {
    Continue,
    Step(ProgramStep),
    Fault,
}

impl ExecutableRunner {
    pub(super) fn apply_expression_resume(
        &mut self,
        resume: ExpressionResume,
        value: CpuStepResult,
    ) -> ExpressionComplete {
        match resume {
            ExpressionResume::RepeatCondition => {
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                if value.is_truthy() {
                    if frame.scoped {
                        self.variables.pop_scope();
                        self.variables.push_scope();
                    }
                    frame.index = 0;
                } else {
                    self.pop_frame();
                }
                ExpressionComplete::Continue
            }
            ExpressionResume::If {
                true_body,
                false_body,
            } => {
                let body = if value.is_truthy() {
                    Some(true_body)
                } else {
                    false_body
                };
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                frame.index += 1;
                if let Some(body) = body {
                    self.push_statement(*body, None, None);
                }
                ExpressionComplete::Continue
            }
            ExpressionResume::While {
                condition,
                body,
                source_span,
            } => {
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                if value.is_truthy() {
                    frame.index += 1;
                    let loop_body = body.map_or_else(
                        || {
                            ExecutableStatement::at(
                                source_span,
                                ExecutableStatementKind::Sequence(vec![]),
                            )
                        },
                        |statement| *statement,
                    );
                    self.push_statement(loop_body, Some(condition), Some(source_span));
                    ExpressionComplete::Continue
                } else {
                    frame.index += 1;
                    ExpressionComplete::Continue
                }
            }
            ExpressionResume::Declare { name, value_type } => {
                self.variables.declare(name, value, value_type);
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::Assign { name } => {
                self.variables.set(&name, value);
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::ExpressionStatement => {
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::DynamicMove => {
                let action = ExecutableAction::Move(value.as_f64());
                if !PendingProgramMotion::is_chunked(action) {
                    // Zero-distance dynamic moves are not pending; advance like a literal move(0).
                    let Some(frame) = self.stack.last_mut() else {
                        return ExpressionComplete::Fault;
                    };
                    frame.index += 1;
                }
                ExpressionComplete::Step(ProgramStep::Action(action))
            }
            ExpressionResume::DynamicRotate => {
                let action = ExecutableAction::Rotate(value.as_f64());
                if !PendingProgramMotion::is_chunked(action) {
                    let Some(frame) = self.stack.last_mut() else {
                        return ExpressionComplete::Fault;
                    };
                    frame.index += 1;
                }
                ExpressionComplete::Step(ProgramStep::Action(action))
            }
            ExpressionResume::DynamicDump => {
                let Some(frame) = self.stack.last_mut() else {
                    return ExpressionComplete::Fault;
                };
                frame.index += 1;
                ExpressionComplete::Step(ProgramStep::Action(ExecutableAction::Dump(
                    value.as_i64() as i32,
                )))
            }
            ExpressionResume::Return => {
                let Some(return_type) = self.current_function_return_type() else {
                    return ExpressionComplete::Fault;
                };
                match self.complete_function_return(value.coerce_to(return_type)) {
                    StepOutcome::Continue => ExpressionComplete::Continue,
                    StepOutcome::Fault => ExpressionComplete::Fault,
                    StepOutcome::Cpu => ExpressionComplete::Continue,
                    StepOutcome::Done => ExpressionComplete::Step(ProgramStep::Done),
                    StepOutcome::Action(action) => {
                        ExpressionComplete::Step(ProgramStep::Action(action))
                    }
                }
            }
        }
    }
}
