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
            .map(|json| json.replace('<', "\\u003c"))
            .unwrap_or_else(|_| "{}".to_string())
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

/// Soft cap for `RallyResult.resultData` so INSERT stays under typical
/// `max_allowed_packet` (16 MiB) with SQL framing headroom.
pub const MAX_PERSISTED_RESULT_DATA_BYTES: usize = 12 * 1024 * 1024;

/// Shrink animation JSON until it fits `max_bytes`, preferring motion replay over
/// CPU-debug fidelity. Oversized payloads otherwise fail persist and leave mining
/// runs unclaimable.
pub fn fit_result_data_for_persist(result_data: &str, max_bytes: usize) -> String {
    if result_data.len() <= max_bytes {
        return result_data.to_string();
    }

    let Ok(mut payload) = AnimationPayload::parse(result_data) else {
        return minimal_persist_stub(0, 0).to_embedded_json_capped(max_bytes);
    };

    strip_cpu_locals(&mut payload);
    let stripped_locals = payload.to_embedded_json();
    if stripped_locals.len() <= max_bytes {
        return stripped_locals;
    }

    strip_cpu_steps(&mut payload);
    let stripped_cpu = payload.to_embedded_json();
    if stripped_cpu.len() <= max_bytes {
        return stripped_cpu;
    }

    clear_motion_detail(&mut payload);
    let cleared = payload.to_embedded_json();
    if cleared.len() <= max_bytes {
        return cleared;
    }

    minimal_persist_stub(payload.ground.size_x, payload.ground.size_y)
        .to_embedded_json_capped(max_bytes)
}

impl AnimationPayload {
    fn to_embedded_json_capped(&self, max_bytes: usize) -> String {
        let json = self.to_embedded_json();
        if json.len() <= max_bytes {
            json
        } else {
            // Last-resort ASCII stub that stays under any positive budget.
            let stub = "{}";
            if stub.len() <= max_bytes {
                stub.to_string()
            } else {
                String::new()
            }
        }
    }
}

fn strip_cpu_locals(payload: &mut AnimationPayload) {
    for robot in &mut payload.robots.robot {
        for location in &mut robot.locations {
            if let Some(cpu) = location.cpu.as_mut() {
                for step in cpu {
                    step.vs = None;
                }
            }
        }
    }
}

fn strip_cpu_steps(payload: &mut AnimationPayload) {
    for robot in &mut payload.robots.robot {
        for location in &mut robot.locations {
            if let Some(cpu) = location.cpu.take()
                && location.source_line.is_none()
            {
                location.source_line = cpu.first().map(|step| step.l);
            }
        }
    }
}

fn clear_motion_detail(payload: &mut AnimationPayload) {
    for robot in &mut payload.robots.robot {
        robot.locations.clear();
    }
    payload.ground.positions.clear();
}

fn minimal_persist_stub(size_x: usize, size_y: usize) -> AnimationPayload {
    AnimationPayload {
        v: ANIMATION_PAYLOAD_VERSION,
        robots: AnimationRobots { robot: Vec::new() },
        ground: AnimationGround {
            size_x,
            size_y,
            positions: Vec::new(),
        },
        ore_types: BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload_with_cpu(locations: usize, cpu_steps: usize) -> String {
        let mut cpu = Vec::with_capacity(cpu_steps);
        for index in 0..cpu_steps {
            let mut vs = BTreeMap::new();
            vs.insert(
                format!("var{index}"),
                AnimationCpuStepResult {
                    k: AnimationCpuStepResultKind::Int,
                    v: index as f64,
                },
            );
            cpu.push(AnimationCpuStep {
                l: (index as u16) + 1,
                c: Some(1),
                e: Some(4),
                r: Some(AnimationCpuStepResult {
                    k: AnimationCpuStepResultKind::Int,
                    v: 1.0,
                }),
                vs: Some(vs),
            });
        }

        let location_count = locations;
        let location_rows: Vec<AnimationLocation> = (0..location_count)
            .map(|turn| AnimationLocation {
                x: Some(turn as f64),
                y: Some(0.0),
                o: Some(90),
                cpu: Some(cpu.clone()),
                ..AnimationLocation::default()
            })
            .collect();

        AnimationPayload {
            v: ANIMATION_PAYLOAD_VERSION,
            robots: AnimationRobots {
                robot: vec![AnimationRobot {
                    robotnr: 0,
                    x: 0.0,
                    y: 0.0,
                    o: 0,
                    ore_a: 0,
                    ore_b: 0,
                    ore_c: 0,
                    size: 1.0,
                    maxore: 100,
                    maxturns: location_count as i32,
                    cpuspeed: 72,
                    depot_max_a: None,
                    depot_max_b: None,
                    depot_max_c: None,
                    depot_a: None,
                    depot_b: None,
                    depot_c: None,
                    home_x: None,
                    home_y: None,
                    home_size: None,
                    locations: location_rows,
                }],
            },
            ground: AnimationGround {
                size_x: 10,
                size_y: 10,
                positions: Vec::new(),
            },
            ore_types: BTreeMap::new(),
        }
        .to_embedded_json()
    }

    #[test]
    fn fit_result_data_keeps_payload_under_budget_by_stripping_cpu() {
        let original = sample_payload_with_cpu(40, 20);
        assert!(
            original.len() > 8_000,
            "fixture should be oversized for a tight budget, got {}",
            original.len()
        );

        let fitted = fit_result_data_for_persist(&original, 8_000);
        assert!(
            fitted.len() <= 8_000,
            "fitted payload must fit budget, got {}",
            fitted.len()
        );

        let payload = AnimationPayload::parse(&fitted).expect("fitted JSON must parse");
        assert_eq!(payload.robots.robot.len(), 1);
        assert!(
            !payload.robots.robot[0]
                .locations
                .iter()
                .any(|location| location.cpu.as_ref().is_some_and(|cpu| !cpu.is_empty())),
            "CPU debug arrays must be stripped when over budget"
        );
        assert!(
            !payload.robots.robot[0].locations.is_empty(),
            "motion locations should remain after CPU strip"
        );
    }

    #[test]
    fn fit_result_data_leaves_small_payload_unchanged() {
        let original = sample_payload_with_cpu(2, 1);
        let fitted = fit_result_data_for_persist(&original, MAX_PERSISTED_RESULT_DATA_BYTES);
        assert_eq!(fitted, original);
    }

    #[test]
    fn fit_result_data_falls_back_to_empty_motion_when_still_too_large() {
        let original = sample_payload_with_cpu(80, 30);
        // Budget below any motion-preserving shrink of this fixture.
        let fitted = fit_result_data_for_persist(&original, 400);
        assert!(fitted.len() <= 400, "got {}", fitted.len());
        let payload = AnimationPayload::parse(&fitted).expect("fitted JSON must parse");
        assert!(
            payload
                .robots
                .robot
                .iter()
                .all(|robot| robot.locations.is_empty())
                || payload.robots.robot.is_empty(),
            "motion detail must be dropped when still over budget"
        );
    }

    #[test]
    fn golden_payload_v2_deserializes() {
        let json = include_str!("../../resources/rally_animation/golden_payload_v2.json");
        let payload = AnimationPayload::parse(json).expect("golden payload should parse");
        assert_eq!(payload.v, ANIMATION_PAYLOAD_VERSION);
        assert_eq!(payload.robots.robot.len(), 1);
        assert_eq!(payload.ground.size_x, 4);
        assert_eq!(payload.ground.size_y, 4);
    }
}
