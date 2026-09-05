use super::super::helpers::*;
use crate::*;

#[test]
fn animation_data_records_sticky_cpu_span_for_multi_cycle_move() {
    let program = seeded_program("move(3);");
    let ground = Ground::new(8, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 6;
    spec.cpu_speed = 72;
    spec.forward_speed = 1.0;

    let mut simulation = Simulation::new(
        ground,
        6,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    let sticky_cpu_locations = locations
        .iter()
        .filter(|location| {
            location
                .get("cpu")
                .and_then(|value| value.as_array())
                .is_some_and(|cpu| {
                    cpu.len() == 1
                        && cpu[0].get("c").is_some()
                        && cpu[0].get("e").is_some()
                        && cpu[0].get("r").is_none()
                })
        })
        .count();
    assert!(
        sticky_cpu_locations >= 1,
        "multi-cycle move should record sticky cpu spans without r: {data}"
    );

    // Final completion step of statement move(3) should record accumulated travel in `r`.
    let mut saw_completion_travel = false;
    for location in locations {
        if let Some(cpu) = location.get("cpu").and_then(|value| value.as_array()) {
            for step in cpu {
                if let Some(v) = step
                    .get("r")
                    .and_then(|r| r.get("v"))
                    .and_then(|v| v.as_f64())
                    && (v - 3.0).abs() < 1e-6
                {
                    saw_completion_travel = true;
                }
            }
        }
    }
    assert!(
        saw_completion_travel,
        "statement move(3) completion should record travel r≈3: {data}"
    );
}

#[test]
fn animation_data_dynamic_move_issue_and_sticky_use_call_span() {
    let program = seeded_program("double d = 3;\nmove(d);");
    let ground = Ground::new(8, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 6;
    spec.cpu_speed = 72;
    spec.forward_speed = 1.0;

    let mut simulation = Simulation::new(
        ground,
        6,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    let mut saw_call_span = false;
    let mut saw_travel = false;
    for location in locations {
        if let Some(cpu) = location.get("cpu").and_then(|value| value.as_array()) {
            for step in cpu {
                let line = step.get("l").and_then(|v| v.as_u64());
                let start = step.get("c").and_then(|v| v.as_u64());
                let end = step.get("e").and_then(|v| v.as_u64());
                if line == Some(2)
                    && let (Some(c), Some(e)) = (start, end)
                    && e > c + 1
                {
                    // Identifier-only "d" would be a single column; call span is wider.
                    saw_call_span = true;
                }
                if let Some(v) = step
                    .get("r")
                    .and_then(|r| r.get("v"))
                    .and_then(|v| v.as_f64())
                    && (v - 3.0).abs() < 1e-6
                {
                    saw_travel = true;
                }
            }
        }
    }
    assert!(
        saw_call_span,
        "dynamic move(d) issue/sticky should highlight the call span: {data}"
    );
    assert!(
        saw_travel,
        "dynamic move(d) completion should record travel r≈3: {data}"
    );
}

#[test]
fn animation_data_done_restart_does_not_reseed_pre_done_highlight() {
    // Finite program: declare then Done. Under high CPU the loop restarts in-cycle;
    // pre-Done steps must not reseed last_cpu_highlight after Done clears it.
    let program = seeded_program("int x = 1;");
    let ground = Ground::new(4, 4);

    let mut short_spec = RobotSpec::test_robot();
    short_spec.max_turns = 1;
    short_spec.cpu_speed = 72;

    let mut long_spec = RobotSpec::test_robot();
    long_spec.max_turns = 4;
    long_spec.cpu_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![
            ScriptedRobot::from_executable_program(short_spec, &program),
            ScriptedRobot::from_executable_program(long_spec, &seeded_program("{}")),
        ],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("done-restart animation should finish");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    // Battery cycles after max_turns must not sticky-rematch the declare span from a
    // cleared Done seed (would show as sticky cpu without a matching live statement).
    let battery_with_sticky = locations.iter().any(|location| {
        location.get("s").and_then(|s| s.as_str()) == Some("battery")
            && location
                .get("cpu")
                .and_then(|value| value.as_array())
                .is_some_and(|cpu| !cpu.is_empty())
    });
    assert!(
        !battery_with_sticky,
        "Done restart must not leave a sticky seed for battery cycles: {data}"
    );
}

#[test]
fn animation_data_records_battery_sticky_cpu_without_r() {
    let program = seeded_program("mine();");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 40);

    let mut short_spec = RobotSpec::test_robot();
    short_spec.max_turns = 1;
    short_spec.cpu_speed = 72;
    short_spec.mining_speed = 4;

    // A longer-lived peer keeps total_moves above the short robot's battery so
    // depleted cycles are actually simulated (run caps total_moves to max max_turns).
    let mut long_spec = RobotSpec::test_robot();
    long_spec.max_turns = 4;
    long_spec.cpu_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![
            ScriptedRobot::from_executable_program(short_spec, &program),
            ScriptedRobot::from_executable_program(long_spec, &seeded_program("{}")),
        ],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("battery sticky animation should finish");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    let battery_sticky = locations.iter().any(|location| {
        location.get("s").and_then(|s| s.as_str()) == Some("battery")
            && location
                .get("cpu")
                .and_then(|value| value.as_array())
                .is_some_and(|cpu| {
                    !cpu.is_empty() && cpu.iter().all(|step| step.get("r").is_none())
                })
    });
    assert!(
        battery_sticky,
        "battery cycles should keep sticky cpu without r: {data}"
    );
}
