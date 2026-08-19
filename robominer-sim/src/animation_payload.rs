//! Typed wire format for rally animation payloads (`RallyResult.resultData`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ANIMATION_PAYLOAD_VERSION;

/// Versioned animation document stored as JSON and loaded by the web viewer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationPayload {
    pub v: u32,
    pub robots: AnimationRobots,
    pub ground: AnimationGround,
    #[serde(rename = "oreTypes")]
    pub ore_types: BTreeMap<String, AnimationOreType>,
}

impl AnimationPayload {
    pub fn parse(result_data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(result_data)
    }

    pub fn to_embedded_json(&self) -> String {
        // Prevent `</script>` breakout when the JSON is embedded in HTML.
        serde_json::to_string(self)
            .expect("animation payload should serialize")
            .replace('<', "\\u003c")
    }

    pub fn with_version(mut self) -> Self {
        self.v = ANIMATION_PAYLOAD_VERSION;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationRobots {
    pub robot: Vec<AnimationRobot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationRobot {
    pub robotnr: usize,
    pub x: f64,
    pub y: f64,
    pub o: i32,
    #[serde(rename = "A")]
    pub ore_a: i32,
    #[serde(rename = "B")]
    pub ore_b: i32,
    #[serde(rename = "C")]
    pub ore_c: i32,
    pub size: f64,
    pub maxore: i32,
    pub maxturns: i32,
    #[serde(default)]
    pub cpuspeed: i32,
    #[serde(rename = "depotMaxA", skip_serializing_if = "Option::is_none", default)]
    pub depot_max_a: Option<i32>,
    #[serde(rename = "depotMaxB", skip_serializing_if = "Option::is_none", default)]
    pub depot_max_b: Option<i32>,
    #[serde(rename = "depotMaxC", skip_serializing_if = "Option::is_none", default)]
    pub depot_max_c: Option<i32>,
    #[serde(rename = "DA", skip_serializing_if = "Option::is_none", default)]
    pub depot_a: Option<i32>,
    #[serde(rename = "DB", skip_serializing_if = "Option::is_none", default)]
    pub depot_b: Option<i32>,
    #[serde(rename = "DC", skip_serializing_if = "Option::is_none", default)]
    pub depot_c: Option<i32>,
    #[serde(rename = "homeX", skip_serializing_if = "Option::is_none", default)]
    pub home_x: Option<usize>,
    #[serde(rename = "homeY", skip_serializing_if = "Option::is_none", default)]
    pub home_y: Option<usize>,
    #[serde(rename = "homeSize", skip_serializing_if = "Option::is_none", default)]
    pub home_size: Option<usize>,
    pub locations: Vec<AnimationLocation>,
}

/// One turn sample. Unchanged fields may be omitted (delta compression).
///
/// # Source highlighting
///
/// Emit at most one of these channels per location (`l` XOR non-empty `cpu`):
/// - **`cpu`**: ordered micro-steps for token-level replay scrubbing. Prefer this when
///   present. Continuation cycles for multi-cycle `move`/`rotate` should repeat the
///   issuing step's `{l,c,e,vs}` with `r` omitted (sticky cpu), not bare `l`.
/// - **`l` (`source_line`)**: legacy/fallback single 1-based line when there are no CPU
///   micro-steps (older payloads, program entry, etc.).
///
/// # Pose vs CPU clock
///
/// `locations[m]` stores the pose **after** turn `m`. The `cpu` entries on that
/// sample are the micro-steps that produced the motion animated during clock segment
/// `[m-1, m)` (viewer highlights destination-sample CPUs while interpolating into `m`).
///
/// # CPU step fields
///
/// - `c`/`e`: 1-based half-open `[c, e)` source columns; omit when unknown.
/// - `r`: typed return `{k,v}` when the micro-step completed a value; omit while awaiting
///   physics (issued `move`/`rotate`) or on sticky continuation steps.
/// - `vs`: full visible-locals snapshot (not a delta); omit when empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AnimationLocation {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub o: Option<i32>,
    #[serde(rename = "A", skip_serializing_if = "Option::is_none", default)]
    pub ore_a: Option<i32>,
    #[serde(rename = "B", skip_serializing_if = "Option::is_none", default)]
    pub ore_b: Option<i32>,
    #[serde(rename = "C", skip_serializing_if = "Option::is_none", default)]
    pub ore_c: Option<i32>,
    #[serde(rename = "DA", skip_serializing_if = "Option::is_none", default)]
    pub depot_a: Option<i32>,
    #[serde(rename = "DB", skip_serializing_if = "Option::is_none", default)]
    pub depot_b: Option<i32>,
    #[serde(rename = "DC", skip_serializing_if = "Option::is_none", default)]
    pub depot_c: Option<i32>,
    #[serde(rename = "a", skip_serializing_if = "Option::is_none", default)]
    pub action_index: Option<u8>,
    #[serde(rename = "l", skip_serializing_if = "Option::is_none", default)]
    pub source_line: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cpu: Option<Vec<AnimationCpuStep>>,
    #[serde(rename = "s", skip_serializing_if = "Option::is_none", default)]
    pub status: Option<String>,
    #[serde(rename = "t", skip_serializing_if = "Option::is_none", default)]
    pub time_fraction: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationCpuStep {
    pub l: u16,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub c: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub e: Option<u16>,
    /// Typed return value for this micro-step.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub r: Option<AnimationCpuStepResult>,
    /// Visible locals at this micro-step (name → typed `{k,v}`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vs: Option<BTreeMap<String, AnimationCpuStepResult>>,
}

/// Wire display kind: `b` bool, `i` int, `f` float (AST `Double` ≡ float).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationCpuStepResultKind {
    #[serde(rename = "b")]
    Bool,
    #[serde(rename = "i")]
    Int,
    #[serde(rename = "f")]
    Float,
}

impl From<robominer_program::CpuStepResultKind> for AnimationCpuStepResultKind {
    fn from(kind: robominer_program::CpuStepResultKind) -> Self {
        match kind {
            robominer_program::CpuStepResultKind::Bool => Self::Bool,
            robominer_program::CpuStepResultKind::Int => Self::Int,
            robominer_program::CpuStepResultKind::Float => Self::Float,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationCpuStepResult {
    pub k: AnimationCpuStepResultKind,
    pub v: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationGround {
    #[serde(rename = "sizeX")]
    pub size_x: usize,
    #[serde(rename = "sizeY")]
    pub size_y: usize,
    pub positions: Vec<AnimationGroundPosition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationGroundPosition {
    pub x: usize,
    pub y: usize,
    pub c: Vec<AnimationGroundChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AnimationGroundChange {
    #[serde(rename = "t", skip_serializing_if = "Option::is_none", default)]
    pub time: Option<i32>,
    #[serde(rename = "A", skip_serializing_if = "Option::is_none", default)]
    pub ore_a: Option<i32>,
    #[serde(rename = "B", skip_serializing_if = "Option::is_none", default)]
    pub ore_b: Option<i32>,
    #[serde(rename = "C", skip_serializing_if = "Option::is_none", default)]
    pub ore_c: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationOreType {
    pub id: i64,
    pub max: i32,
}
