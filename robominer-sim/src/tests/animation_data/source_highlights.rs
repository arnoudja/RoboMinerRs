use super::super::helpers::*;
use crate::*;

#[test]
fn animation_data_records_source_line_for_program_actions() {
    let program = seeded_program("scan();\nmine();");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 8;
    spec.scan_time = 2;
    spec.cpu_speed = 72;
    spec.mining_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        8,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    assert!(
        locations.iter().any(|location| {
            animation_location_highlight_line(location).is_some_and(|line| line == 1 || line == 2)
        }),
        "program animation should include source highlights: {data}"
    );
    assert!(
        locations.iter().any(|location| {
            location.get("a").and_then(|value| value.as_u64()) == Some(6)
                && animation_location_highlight_line(location).is_some()
        }),
        "mine cycles should include a source highlight: {data}"
    );
    assert!(
        data.contains(r#""cpu":"#),
        "program animation should include cpu micro-steps: {data}"
    );
    for location in locations {
        let has_l = location.get("l").is_some();
        let has_cpu = location
            .get("cpu")
            .and_then(|value| value.as_array())
            .is_some_and(|cpu| !cpu.is_empty());
        assert!(
            !(has_l && has_cpu),
            "location must not duplicate sticky `l` and `cpu` highlights: {location}"
        );
    }
    assert!(
        !data.contains("src:"),
        "program source must not be embedded in shared animation data: {data}"
    );
}

#[test]
fn animation_data_attributes_while_recheck_to_while_line() {
    let program = seeded_program("while (move(1) >= 1)\n{\nmine();\n}");
    let mut ground = Ground::new(6, 6);
    ground.at_mut(0, 0).add_ore(0, 40);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 8;
    spec.cpu_speed = 72;
    spec.forward_speed = 1.0;
    spec.mining_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        8,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations");

    let mut saw_mine_on_body_line = false;
    let mut saw_move_on_while_line_after_mine = false;
    let mut saw_mine = false;

    for location in locations {
        let action = location.get("a").and_then(|v| v.as_u64());
        let line = animation_location_highlight_line(location);
        if animation_location_cpu_lines(location).contains(&3) {
            saw_mine_on_body_line = true;
        }
        if action == Some(6) {
            saw_mine = true;
        }
        if saw_mine && action == Some(2) && line == Some(1) {
            saw_move_on_while_line_after_mine = true;
            break;
        }
    }

    assert!(
        saw_mine_on_body_line,
        "expected mine on body line 3 in {data}"
    );
    assert!(
        saw_move_on_while_line_after_mine,
        "expected later move cycles to attribute to while line 1 in {data}"
    );
}

#[test]
fn animation_data_cpu_span_matches_bool_literal_after_scan() {
    let program = seeded_program("scan();\nbool found = false;");
    let ground = Ground::new(4, 4);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;
    spec.scan_time = 5;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");

    let mut steps = Vec::new();
    for location in payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations")
    {
        if let Some(cpu) = location.get("cpu").and_then(|value| value.as_array()) {
            for step in cpu {
                steps.push(step.clone());
            }
        }
    }

    let bool_step = steps.iter().find(|step| {
        step.get("r")
            .and_then(|result| result.get("k"))
            .and_then(|kind| kind.as_str())
            == Some("b")
    });
    let bool_step = bool_step.unwrap_or_else(|| panic!("expected a bool return step: {data}"));
    assert_eq!(
        bool_step.get("l").and_then(|value| value.as_u64()),
        Some(2),
        "bool literal should highlight line 2, got {bool_step:?} in {data}"
    );
    // `bool found = false;` — `false` starts after "bool found = ".
    let start = bool_step.get("c").and_then(|value| value.as_u64());
    let end = bool_step.get("e").and_then(|value| value.as_u64());
    assert!(
        start.is_some_and(|c| c > 1) && end.is_some_and(|e| e > start.unwrap()),
        "bool literal should have a token span, got c={start:?} e={end:?} in {data}"
    );
}
