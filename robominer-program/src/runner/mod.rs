mod expression_eval;
mod step;

use crate::cpu_step_result::CpuStepResult;
use crate::pending_program_motion::{PendingProgramMotion, ProgramMotionCompletion};

use crate::types::*;
use std::collections::BTreeMap;

use expression_eval::{OngoingExpressionEval, RuntimeVariables};

/// Maximum nested user-function call depth before the runner faults.
pub(crate) const MAX_CALL_DEPTH: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExecutionFrame {
    statements: Vec<ExecutableStatement>,
    index: usize,
    repeat_condition: Option<ExecutableExpression>,
    /// Source location of the while/do that owns [`Self::repeat_condition`].
    repeat_source_span: Option<SourceSpan>,
    scoped: bool,
    /// True when this frame is a user-function body (triggers return on fall-through).
    is_function_call: bool,
    /// Return type of the active function call; set when [`Self::is_function_call`].
    call_return_type: Option<ValueType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableRunner {
    stack: Vec<ExecutionFrame>,
    variables: RuntimeVariables,
    functions: BTreeMap<String, ExecutableFunction>,
    call_depth: usize,
    /// Suspended expression evaluations waiting for a user-call return (stack).
    suspended_expression_evals: Vec<OngoingExpressionEval>,
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
    Fault,
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
                is_function_call: false,
                call_return_type: None,
            }],
            variables: RuntimeVariables::default(),
            functions: program.functions,
            call_depth: 0,
            suspended_expression_evals: Vec::new(),
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
        self.variables.get_typed(name).as_f64()
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
    /// CPU cycle will run, so replay highlighting follows evaluation inside a statement.
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
                ProgramStep::Done | ProgramStep::Fault => return None,
                ProgramStep::Cpu => {}
            }
        }
    }

    /// Clear expression/pending/call state and report a recoverable runner fault.
    ///
    /// Used for expected program faults (e.g. call-depth overflow). Prefer
    /// [`Self::abort_with_fault`] for broken runner invariants (debug-asserted).
    pub(crate) fn fault_program(&mut self) -> StepOutcome {
        self.expression_eval = None;
        self.suspended_expression_evals.clear();
        self.call_depth = 0;
        self.clear_pending_action_handshake();
        self.set_active_source(None);
        self.last_step_result = None;
        self.last_step_span = None;
        StepOutcome::Fault
    }

    /// Clear expression/pending state and report a recoverable runner fault.
    pub(crate) fn abort_with_fault(&mut self) -> StepOutcome {
        debug_assert!(false, "program runner invariant failed");
        self.fault_program()
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
                StepOutcome::Fault => {
                    break ProgramStep::Fault;
                }
            }
        };

        context.action_result = action_result;
        step
    }

    pub(super) fn queue_pending_action(&mut self, action: ExecutableAction) -> ExecutableAction {
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

    pub(super) fn start_pending_program_motion(
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

    pub(super) fn push_statement(
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

    pub(super) fn push_frame(
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
            is_function_call: false,
            call_return_type: None,
        });
    }

    /// Push a scoped frame for a user-function body and bind parameters in that scope.
    pub(super) fn push_function_call_frame(
        &mut self,
        body: Vec<ExecutableStatement>,
        return_type: ValueType,
        params: &[(String, ValueType, CpuStepResult)],
    ) {
        self.variables.push_scope();
        for (name, value_type, value) in params {
            self.variables.declare(name.clone(), *value, *value_type);
        }
        self.stack.push(ExecutionFrame {
            statements: body,
            index: 0,
            repeat_condition: None,
            repeat_source_span: None,
            scoped: true,
            is_function_call: true,
            call_return_type: Some(return_type),
        });
    }

    pub(super) fn current_function_return_type(&self) -> Option<ValueType> {
        self.stack
            .iter()
            .rev()
            .find_map(|frame| frame.call_return_type)
    }

    /// Pop frames until the active function-call frame is gone, restore the suspended
    /// caller expression, and push `value` as the call result.
    pub(super) fn complete_function_return(&mut self, value: CpuStepResult) -> StepOutcome {
        loop {
            let Some(frame) = self.stack.last() else {
                return self.abort_with_fault();
            };
            let is_call = frame.is_function_call;
            self.pop_frame();
            if is_call {
                break;
            }
        }

        if self.call_depth == 0 {
            return self.abort_with_fault();
        }
        self.call_depth -= 1;

        let Some(mut eval) = self.suspended_expression_evals.pop() else {
            return self.abort_with_fault();
        };
        eval.values.push(value);
        eval.index += 1;
        self.expression_eval = Some(eval);
        StepOutcome::Continue
    }

    pub(super) fn set_active_source(&mut self, span: Option<SourceSpan>) {
        self.active_source_line = span.map(|span| span.line);
        self.active_source_span = span;
    }

    pub(super) fn pop_frame(&mut self) {
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
