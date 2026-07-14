use super::schedule::Truthy;
use super::super::ExecutableRunner;
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
    },
    Declare { name: String },
    Assign { name: String },
    ExpressionStatement,
    DynamicMove,
    DynamicRotate,
    DynamicDump,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExpressionComplete {
    Continue,
    Step(ProgramStep),
}

impl ExecutableRunner {
    pub(super) fn apply_expression_resume(
        &mut self,
        resume: ExpressionResume,
        value: f64,
    ) -> ExpressionComplete {
        match resume {
            ExpressionResume::RepeatCondition => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("repeat condition requires an active frame");
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
                let frame = self
                    .stack
                    .last_mut()
                    .expect("if condition requires an active frame");
                frame.index += 1;
                if let Some(body) = body {
                    self.push_statement(*body, None);
                }
                ExpressionComplete::Continue
            }
            ExpressionResume::While { condition, body } => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("while condition requires an active frame");
                if value.is_truthy() {
                    frame.index += 1;
                    let loop_body = body.map_or_else(
                        || ExecutableStatement::Sequence(vec![]),
                        |statement| *statement,
                    );
                    self.push_statement(loop_body, Some(condition));
                    ExpressionComplete::Continue
                } else {
                    frame.index += 1;
                    ExpressionComplete::Continue
                }
            }
            ExpressionResume::Declare { name } => {
                self.variables.declare(name, value);
                let frame = self
                    .stack
                    .last_mut()
                    .expect("declare requires an active frame");
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::Assign { name } => {
                self.variables.set(&name, value);
                let frame = self
                    .stack
                    .last_mut()
                    .expect("assign requires an active frame");
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::ExpressionStatement => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("expression statement requires an active frame");
                frame.index += 1;
                ExpressionComplete::Continue
            }
            ExpressionResume::DynamicMove => {
                ExpressionComplete::Step(ProgramStep::Action(ExecutableAction::Move(value)))
            }
            ExpressionResume::DynamicRotate => {
                ExpressionComplete::Step(ProgramStep::Action(ExecutableAction::Rotate(value)))
            }
            ExpressionResume::DynamicDump => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("dynamic dump requires an active frame");
                frame.index += 1;
                ExpressionComplete::Step(ProgramStep::Action(ExecutableAction::Dump(value as i32)))
            }
        }
    }
}
