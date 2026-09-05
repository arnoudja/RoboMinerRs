use std::collections::BTreeMap;

use crate::MAX_ORE_TYPES;
use crate::animation_payload::{
    AnimationCpuStep, AnimationCpuStepResult, AnimationCpuStepResultKind,
};
use crate::position::Position;

/// Current on-disk / wire format for rally animation payloads stored in
/// `RallyResult.resultData`. Older executable JavaScript rows (`var myRobots = …`)
/// are no longer played by the web viewer.
///
/// Version 2 adds optional per-turn `cpu` arrays of instruction spans,
/// typed step results (`r`), and locals snapshots (`vs`).
pub const ANIMATION_PAYLOAD_VERSION: u32 = 2;

pub struct OreAnimationData {
    pub ore_id: i64,
    pub max_amount: i32,
}

/// One program CPU instruction within a turn (for replay stepping/highlight).
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedCpuStep {
    pub line: u16,
    pub start_col: u16,
    pub end_col: u16,
    pub result: Option<robominer_program::CpuStepResult>,
    pub variables: BTreeMap<String, robominer_program::CpuStepResult>,
}

impl RecordedCpuStep {
    pub fn from_span(span: robominer_program::SourceSpan) -> Option<Self> {
        if !span.is_known() {
            return None;
        }
        Some(Self {
            line: span.line,
            start_col: span.start_col,
            end_col: span.end_col,
            result: None,
            variables: BTreeMap::new(),
        })
    }

    /// True when this step has a token column range (matches [`SourceSpan::has_columns`]).
    pub fn has_columns(&self) -> bool {
        self.line != 0 && self.start_col > 0 && self.end_col > self.start_col
    }

    pub fn with_result(mut self, result: Option<robominer_program::CpuStepResult>) -> Self {
        self.result = result;
        self
    }

    pub fn with_variables(
        mut self,
        variables: BTreeMap<String, robominer_program::CpuStepResult>,
    ) -> Self {
        self.variables = variables;
        self
    }
}

fn animation_cpu_step_result(result: robominer_program::CpuStepResult) -> AnimationCpuStepResult {
    AnimationCpuStepResult {
        k: AnimationCpuStepResultKind::from(result.kind()),
        v: result.wire_f64(),
    }
}

impl From<RecordedCpuStep> for AnimationCpuStep {
    fn from(step: RecordedCpuStep) -> Self {
        let has_columns = step.has_columns();
        let mut entry = AnimationCpuStep {
            l: step.line,
            c: None,
            e: None,
            r: step.result.map(animation_cpu_step_result),
            vs: if step.variables.is_empty() {
                None
            } else {
                Some(
                    step.variables
                        .into_iter()
                        .map(|(name, result)| (name, animation_cpu_step_result(result)))
                        .collect(),
                )
            },
        };
        if has_columns {
            entry.c = Some(step.start_col);
            entry.e = Some(step.end_col);
        }
        entry
    }
}

/// Compact per-cycle status for stuck/idle diagnosis in the replay viewer.
/// Omitted from JSON when the robot is making normal progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobotCycleStatus {
    /// Individual battery / max_turns exhausted; no action this cycle.
    Battery,
    /// Waiting while a scan completes (paired with action index 0).
    Scan,
    /// CPU budget exhausted before an action was chosen.
    Cpu,
    /// `move(0)` / `rotate(0)` (or epsilon-equivalent) mapped to Wait.
    Zero,
    /// Non-zero motion requested but collapsed to Wait — no speed chunk could be issued
    /// (e.g. zero engine speed). Wire status remains `"motion"` for replay compatibility.
    NoChunk,
    /// Requested move ended at the start pose due to map bounds.
    Wall,
    /// Requested move ended at the start pose due to another robot.
    Robot,
    /// Explicit or residual Wait with no more specific cause.
    Wait,
}

impl RobotCycleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Battery => "battery",
            Self::Scan => "scan",
            Self::Cpu => "cpu",
            Self::Zero => "zero",
            Self::NoChunk => "motion",
            Self::Wall => "wall",
            Self::Robot => "robot",
            Self::Wait => "wait",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RobotAnimationStep {
    pub(super) position: Position,
    pub(super) ore: [i32; MAX_ORE_TYPES],
    pub(super) depot: [i32; MAX_ORE_TYPES],
    pub(super) time_fraction: f64,
    /// Optional action index for this cycle (`RobotAction::action_index`, or 0 for scan).
    /// Absent on the initial step and on legacy replays.
    pub(super) action_index: Option<u8>,
    /// Optional 1-based source line when this cycle has no CPU micro-steps (sticky highlight).
    /// Serialized as `l`; omitted when `cpu_steps` is non-empty.
    pub(super) source_line: Option<u16>,
    /// Optional stuck/idle reason for this cycle.
    pub(super) status: Option<RobotCycleStatus>,
    /// Program CPU micro-steps for this cycle; serialized as `cpu` when non-empty.
    pub(super) cpu_steps: Vec<RecordedCpuStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GroundAnimationStep {
    pub(super) time: i32,
    pub(super) ore: [i32; MAX_ORE_TYPES],
}
