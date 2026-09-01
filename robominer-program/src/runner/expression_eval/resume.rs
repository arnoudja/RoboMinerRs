use super::super::ExecutableRunner;
use crate::pending_program_motion::PendingProgramMotion;
use crate::program_value::{ProgramValue, as_f64_for_action_arg};
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
        value: ProgramValue,
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
                let action = ExecutableAction::Move(as_f64_for_action_arg(value));
                if !PendingProgramMotion::is_chunked(action) {
                    let Some(frame) = self.stack.last_mut() else {
                        return ExpressionComplete::Fault;
                    };
                    frame.index += 1;
                }
                ExpressionComplete::Step(ProgramStep::Action(action))
            }
            ExpressionResume::DynamicRotate => {
                let action = ExecutableAction::Rotate(as_f64_for_action_arg(value));
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
                    as_f64_for_action_arg(value) as i32,
                )))
            }
        }
    }
}
