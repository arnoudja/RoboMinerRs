mod expression_eval;

use crate::cpu_step_result::CpuStepResult;
use crate::pending_program_motion::{PendingProgramMotion, ProgramMotionCompletion};

use crate::types::*;

use expression_eval::{ExpressionResume, OngoingExpressionEval, RuntimeVariables};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExecutionFrame {
    statements: Vec<ExecutableStatement>,
    index: usize,
    repeat_condition: Option<ExecutableExpression>,
    /// Source location of the while/do that owns [`Self::repeat_condition`].
    repeat_source_span: Option<SourceSpan>,
    scoped: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableRunner {
    stack: Vec<ExecutionFrame>,
    variables: RuntimeVariables,
    /// Set while the sim must finish a multi-cycle action before the runner advances.
    /// See [`pending_action_protocol`].
    awaits_action_result: bool,
    /// Scan and other non-move actions awaiting a single-cycle result.
    pending_action: Option<ExecutableAction>,
    /// Multi-cycle move/rotate shared by statement and expression paths.
    /// See [`pending_program_motion`] and [`pending_action_protocol`].
    pending_program_motion: Option<PendingProgramMotion>,
    expression_eval: Option<OngoingExpressionEval>,
    /// Source line of the statement most recently entered (survives index advance for
    /// one-shot actions like mine, and multi-cycle pending motion). Refreshed to the
    /// while/do line when a loop re-checks its condition.
    active_source_line: Option<u16>,
    /// Same statement as [`Self::active_source_line`], with columns for replay highlighting.
    active_source_span: Option<SourceSpan>,
    /// Typed result produced by the most recent CPU micro-step, if any.
    last_step_result: Option<CpuStepResult>,
    /// Source span of the work that produced the most recent step (may differ from
    /// [`Self::current_source_span`] taken before `step`, when one call Continues into
    /// expression evaluation).
    last_step_span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StepOutcome {
    Continue,
    Cpu,
    Action(ExecutableAction),
    Done,
}

impl ExecutableRunner {
    pub(crate) fn new(program: ExecutableProgram) -> Self {
        Self {
            stack: vec![ExecutionFrame {
                statements: program.statements,
                index: 0,
                repeat_condition: None,
                repeat_source_span: None,
                scoped: false,
            }],
            variables: RuntimeVariables::default(),
            awaits_action_result: false,
            pending_action: None,
            pending_program_motion: None,
            expression_eval: None,
            active_source_line: None,
            active_source_span: None,
            last_step_result: None,
            last_step_span: None,
        }
    }

    pub fn awaits_action_result(&self) -> bool {
        self.awaits_action_result
    }

    /// Take the typed result of the last [`Self::step`], if that step produced one.
    ///
    /// Callers (e.g. animation recording) must take at most once per `step`. `None` is
    /// expected when the step issued an awaiting action, continued internally, or had
    /// no displayable value.
    pub fn take_last_step_result(&mut self) -> Option<CpuStepResult> {
        self.last_step_result.take()
    }

    /// Take the source span of the work executed by the last [`Self::step`], if known.
    pub fn take_last_step_span(&mut self) -> Option<SourceSpan> {
        self.last_step_span.take()
    }

    pub fn runtime_variable(&self, name: &str) -> f64 {
        self.variables.get(name)
    }

    /// Flattened name → typed value map of currently visible locals.
    pub fn runtime_variables_snapshot(&self) -> std::collections::BTreeMap<String, CpuStepResult> {
        self.variables.snapshot()
    }

    pub fn awaits_scan_result(&self) -> bool {
        self.pending_action == Some(ExecutableAction::AwaitScanResult)
    }

    pub fn pending_scan_start(&self) -> bool {
        matches!(self.pending_action, Some(ExecutableAction::StartScan(_)))
    }

    pub fn has_pending_program_motion(&self) -> bool {
        self.pending_program_motion.is_some()
    }

    /// Drop pending motion/action wait state without consuming an `action_result`.
    /// Used when the sim cannot finish the handshake (e.g. battery expired mid-move).
    pub fn clear_pending_action_handshake(&mut self) {
        self.awaits_action_result = false;
        self.pending_action = None;
        self.pending_program_motion = None;
    }

    /// 1-based source line of the statement currently executing, if any.
    pub fn current_source_line(&self) -> Option<u16> {
        if let Some(span) = self.active_source_span.filter(|span| span.is_known()) {
            return Some(span.line);
        }
        if let Some(line) = self.active_source_line {
            return Some(line);
        }
        let frame = self.stack.last()?;
        if frame.index >= frame.statements.len() {
            return None;
        }
        Some(frame.statements[frame.index].source_line())
    }

    /// Source range of the construct currently executing, if any.
    ///
    /// While an expression is being evaluated this narrows to the sub-expression the next
    /// CPU instruction will run, so replay highlighting follows evaluation inside a statement.
    pub fn current_source_span(&self) -> Option<SourceSpan> {
        if let Some(span) = self
            .expression_eval
            .as_ref()
            .and_then(OngoingExpressionEval::current_span)
            .filter(|span| span.is_known())
        {
            return Some(span);
        }
        if let Some(span) = self.active_source_span.filter(|span| span.is_known()) {
            return Some(span);
        }
        let frame = self.stack.last()?;
        frame
            .statements
            .get(frame.index)
            .map(|statement| statement.source_span)
            .filter(|span| span.is_known())
    }

    pub fn next_action(&mut self, context: &mut ExecutionContext) -> Option<ExecutableAction> {
        loop {
            match self.step(context) {
                ProgramStep::Action(action) => return Some(action),
                ProgramStep::Done => return None,
                ProgramStep::Cpu => {}
            }
        }
    }

    pub fn step(&mut self, context: &mut ExecutionContext) -> ProgramStep {
        self.awaits_action_result = false;
        self.last_step_result = None;
        self.last_step_span = None;
        let mut action_result = context.action_result;

        let step = loop {
            match self.step_with_result(context, &mut action_result) {
                StepOutcome::Continue => continue,
                StepOutcome::Cpu => break ProgramStep::Cpu,
                StepOutcome::Action(action) => {
                    let action = if PendingProgramMotion::is_chunked(action)
                        && self.pending_program_motion.is_none()
                        && self.expression_eval.is_none()
                    {
                        self.start_pending_program_motion(
                            action,
                            ProgramMotionCompletion::Statement,
                        )
                    } else {
                        action
                    };
                    break ProgramStep::Action(action);
                }
                StepOutcome::Done => {
                    self.set_active_source(None);
                    break ProgramStep::Done;
                }
            }
        };

        context.action_result = action_result;
        step
    }

    fn step_with_result(
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
            return StepOutcome::Continue;
        }

        if self
            .stack
            .last()
            .is_some_and(|frame| frame.index >= frame.statements.len())
        {
            self.pop_frame();
            return StepOutcome::Continue;
        }

        let Some(frame) = self.stack.last_mut() else {
            return StepOutcome::Done;
        };

        let statement = frame.statements[frame.index].clone();
        self.active_source_line = Some(statement.source_line());
        self.active_source_span = Some(statement.source_span);

        match statement.kind {
            ExecutableStatementKind::Action(action) => {
                self.last_step_span = Some(statement.source_span);
                if !PendingProgramMotion::is_chunked(action) {
                    frame.index += 1;
                }
                StepOutcome::Action(action)
            }
            ExecutableStatementKind::DynamicAction(action) => {
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
            ExecutableStatementKind::Sequence(statements) => {
                frame.index += 1;
                self.last_step_span = Some(statement.source_span);
                self.push_frame(statements, None, None, true);
                StepOutcome::Cpu
            }
            ExecutableStatementKind::Declare {
                name,
                value_type,
                value,
            } => {
                if let Some(value) = value {
                    self.start_expression_evaluation(
                        value,
                        ExpressionResume::Declare { name, value_type },
                    );
                    StepOutcome::Continue
                } else {
                    self.variables.declare(name, 0.0, value_type);
                    frame.index += 1;
                    self.last_step_span = Some(statement.source_span);
                    StepOutcome::Cpu
                }
            }
            ExecutableStatementKind::Assign { name, value } => {
                self.start_expression_evaluation(value, ExpressionResume::Assign { name });
                StepOutcome::Continue
            }
            ExecutableStatementKind::Expression(expression) => {
                self.start_expression_evaluation(expression, ExpressionResume::ExpressionStatement);
                StepOutcome::Continue
            }
            ExecutableStatementKind::If {
                condition,
                true_body,
                false_body,
            } => {
                self.start_expression_evaluation(
                    condition,
                    ExpressionResume::If {
                        true_body,
                        false_body,
                    },
                );
                StepOutcome::Continue
            }
            ExecutableStatementKind::While {
                condition,
                body,
                is_do_while,
            } => {
                let loop_span = statement.source_span;
                if is_do_while {
                    if let Some(body) = body {
                        frame.index += 1;
                        self.push_statement(*body, Some(condition), Some(loop_span));
                        StepOutcome::Cpu
                    } else if let Some(action) = condition.first_action() {
                        if PendingProgramMotion::is_chunked(action) {
                            StepOutcome::Action(self.start_pending_program_motion(
                                action,
                                ProgramMotionCompletion::Statement,
                            ))
                        } else {
                            StepOutcome::Action(self.queue_pending_action(action))
                        }
                    } else {
                        frame.index += 1;
                        StepOutcome::Cpu
                    }
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
        }
    }

    fn queue_pending_action(&mut self, action: ExecutableAction) -> ExecutableAction {
        match crate::await_kind(action) {
            crate::ActionAwaitKind::Scalar | crate::ActionAwaitKind::ScanStart => {
                self.awaits_action_result = true;
                self.pending_action = Some(action);
            }
            crate::ActionAwaitKind::Motion => {
                // Chunked motion must use start_pending_program_motion, not scalar pending_action.
                debug_assert!(false, "motion action queued via pending_action: {action:?}");
                return self
                    .start_pending_program_motion(action, ProgramMotionCompletion::Expression);
            }
            crate::ActionAwaitKind::None => {
                // Wait-mapped actions never produce action_result; emit without awaiting.
                debug_assert!(
                    false,
                    "queued action that does not produce a result: {action:?}"
                );
            }
        }
        action
    }

    fn start_pending_program_motion(
        &mut self,
        action: ExecutableAction,
        completion: ProgramMotionCompletion,
    ) -> ExecutableAction {
        debug_assert!(
            crate::await_kind(action) == crate::ActionAwaitKind::Motion,
            "start_pending_program_motion requires Motion await kind, got {action:?}"
        );
        self.awaits_action_result = true;
        self.pending_program_motion = Some(PendingProgramMotion::start(action, completion));
        action
    }

    fn push_statement(
        &mut self,
        statement: ExecutableStatement,
        repeat_condition: Option<ExecutableExpression>,
        repeat_source_span: Option<SourceSpan>,
    ) {
        let source_span = statement.source_span;
        match statement.kind {
            ExecutableStatementKind::Sequence(statements) => {
                self.push_frame(statements, repeat_condition, repeat_source_span, true);
            }
            kind => self.push_frame(
                vec![ExecutableStatement::at(source_span, kind)],
                repeat_condition,
                repeat_source_span,
                false,
            ),
        }
    }

    fn push_frame(
        &mut self,
        statements: Vec<ExecutableStatement>,
        repeat_condition: Option<ExecutableExpression>,
        repeat_source_span: Option<SourceSpan>,
        scoped: bool,
    ) {
        if scoped {
            self.variables.push_scope();
        }

        self.stack.push(ExecutionFrame {
            statements,
            index: 0,
            repeat_condition,
            repeat_source_span,
            scoped,
        });
    }

    fn set_active_source(&mut self, span: Option<SourceSpan>) {
        self.active_source_line = span.map(|span| span.line);
        self.active_source_span = span;
    }

    fn pop_frame(&mut self) {
        if let Some(frame) = self.stack.pop()
            && frame.scoped
        {
            self.variables.pop_scope();
        }
    }
}

impl ExecutableProgram {
    pub fn runner(&self) -> ExecutableRunner {
        ExecutableRunner::new(self.clone())
    }
}
