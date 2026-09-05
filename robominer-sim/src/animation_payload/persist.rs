use std::collections::BTreeMap;

use super::types::*;
use crate::ANIMATION_PAYLOAD_VERSION;

/// Soft cap for `RallyResult.resultData` so INSERT stays under typical
/// `max_allowed_packet` (16 MiB) with SQL framing headroom.
pub const MAX_PERSISTED_RESULT_DATA_BYTES: usize = 12 * 1024 * 1024;

/// Shrink animation JSON until it fits `max_bytes`, preferring motion replay over
/// CPU-debug fidelity. Oversized payloads otherwise fail persist and leave mining
/// runs unclaimable.
///
/// Shrink ladder:
/// 1. Sparsify unchanged `cpu[].vs` (viewer carries forward; `vs:{}` clears)
/// 2. Strip all remaining locals
/// 3. Strip CPU micro-steps (line-only fallback)
/// 4. Clear motion detail / minimal stub
pub fn fit_result_data_for_persist(result_data: &str, max_bytes: usize) -> String {
    if result_data.len() <= max_bytes {
        return result_data.to_string();
    }

    let Ok(mut payload) = AnimationPayload::parse(result_data) else {
        return minimal_persist_stub(0, 0).to_embedded_json_capped(max_bytes);
    };

    sparsify_cpu_locals(&mut payload);
    let sparsified = payload.to_embedded_json();
    if sparsified.len() <= max_bytes {
        return sparsified;
    }

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

/// Delta-compress `cpu[].vs` within each robot:
/// - omit when equal to the previous emitted snapshot (viewer carries forward)
/// - emit `Some({})` when locals become empty after a non-empty snapshot (clear carry)
/// - leave leading empty/`None` as `None` (nothing to clear)
pub(crate) fn sparsify_cpu_locals(payload: &mut AnimationPayload) {
    for robot in &mut payload.robots.robot {
        let mut last_vs: Option<BTreeMap<String, AnimationCpuStepResult>> = None;
        for location in &mut robot.locations {
            if let Some(cpu) = location.cpu.as_mut() {
                for step in cpu {
                    match step.vs.take() {
                        None => {
                            if last_vs.as_ref().is_some_and(|vs| !vs.is_empty()) {
                                step.vs = Some(BTreeMap::new());
                                last_vs = Some(BTreeMap::new());
                            }
                        }
                        Some(vs) => {
                            if last_vs.as_ref() == Some(&vs) {
                                // Unchanged — omit; viewer keeps prior snapshot.
                                step.vs = None;
                            } else {
                                last_vs = Some(vs.clone());
                                step.vs = Some(vs);
                            }
                        }
                    }
                }
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
    fn fit_result_data_sparsifies_unchanged_locals_before_stripping() {
        // Sticky multi-cycle motion repeats the same locals snapshot; sparsify should
        // omit unchanged `vs` and keep the first keyframe instead of stripping all.
        let mut vs = BTreeMap::new();
        vs.insert(
            "found".to_string(),
            AnimationCpuStepResult {
                k: AnimationCpuStepResultKind::Bool,
                v: 0.0,
            },
        );
        let sticky_step = AnimationCpuStep {
            l: 2,
            c: Some(12),
            e: Some(34),
            r: None,
            vs: Some(vs),
        };
        let locations: Vec<AnimationLocation> = (0..40)
            .map(|turn| AnimationLocation {
                x: Some(turn as f64),
                y: Some(0.0),
                o: Some(90),
                cpu: Some(vec![sticky_step.clone()]),
                ..AnimationLocation::default()
            })
            .collect();
        let original = AnimationPayload {
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
                    maxturns: locations.len() as i32,
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
                    locations,
                }],
            },
            ground: AnimationGround {
                size_x: 10,
                size_y: 10,
                positions: Vec::new(),
            },
            ore_types: BTreeMap::new(),
        }
        .to_embedded_json();

        assert!(
            original.len() > 2_000,
            "fixture should be oversized for a tight budget, got {}",
            original.len()
        );

        // Budget just under full payload: sparsify must fit without total strip.
        let fitted = fit_result_data_for_persist(&original, original.len() - 1);
        assert!(
            fitted.len() < original.len(),
            "sparsified payload must fit under budget, got {} vs {}",
            fitted.len(),
            original.len() - 1
        );
        assert!(
            fitted.len() < original.len(),
            "sparsify should shrink repeated sticky locals"
        );

        let payload = AnimationPayload::parse(&fitted).expect("fitted JSON must parse");
        let robot = &payload.robots.robot[0];
        let first_vs = robot.locations[0]
            .cpu
            .as_ref()
            .and_then(|cpu| cpu.first())
            .and_then(|step| step.vs.as_ref());
        assert!(
            first_vs.is_some_and(|vs| vs.contains_key("found")),
            "first keyframe locals must remain after sparsify: {fitted}"
        );
        let omitted_later = robot.locations.iter().skip(1).any(|location| {
            location
                .cpu
                .as_ref()
                .and_then(|cpu| cpu.first())
                .is_some_and(|step| step.vs.is_none())
        });
        assert!(
            omitted_later,
            "unchanged sticky locals should be omitted after sparsify: {fitted}"
        );
    }

    #[test]
    fn sparsify_cpu_locals_emits_empty_object_to_clear_carry() {
        let mut vs = BTreeMap::new();
        vs.insert(
            "x".to_string(),
            AnimationCpuStepResult {
                k: AnimationCpuStepResultKind::Int,
                v: 1.0,
            },
        );
        let mut payload = AnimationPayload {
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
                    maxturns: 2,
                    cpuspeed: 4,
                    depot_max_a: None,
                    depot_max_b: None,
                    depot_max_c: None,
                    depot_a: None,
                    depot_b: None,
                    depot_c: None,
                    home_x: None,
                    home_y: None,
                    home_size: None,
                    locations: vec![
                        AnimationLocation {
                            cpu: Some(vec![AnimationCpuStep {
                                l: 1,
                                c: Some(1),
                                e: Some(2),
                                r: None,
                                vs: Some(vs),
                            }]),
                            ..AnimationLocation::default()
                        },
                        AnimationLocation {
                            // Empty locals (omit on wire today) after a prior snapshot.
                            cpu: Some(vec![AnimationCpuStep {
                                l: 1,
                                c: Some(1),
                                e: Some(2),
                                r: None,
                                vs: None,
                            }]),
                            ..AnimationLocation::default()
                        },
                    ],
                }],
            },
            ground: AnimationGround {
                size_x: 2,
                size_y: 2,
                positions: Vec::new(),
            },
            ore_types: BTreeMap::new(),
        };

        sparsify_cpu_locals(&mut payload);

        let clear = payload.robots.robot[0].locations[1]
            .cpu
            .as_ref()
            .expect("cpu")
            .first()
            .expect("step")
            .vs
            .as_ref()
            .expect("empty locals after non-empty must be explicit vs:{{}}");
        assert!(clear.is_empty(), "clear sentinel must be an empty map");
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
        let json = include_str!("../../../resources/rally_animation/golden_payload_v2.json");
        let payload = AnimationPayload::parse(json).expect("golden payload should parse");
        assert_eq!(payload.v, ANIMATION_PAYLOAD_VERSION);
        assert_eq!(payload.robots.robot.len(), 1);
        assert_eq!(payload.ground.size_x, 4);
        assert_eq!(payload.ground.size_y, 4);
    }
}
