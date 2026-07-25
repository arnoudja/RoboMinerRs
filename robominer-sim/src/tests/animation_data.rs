// Unit tests own animation wire semantics; rally goldens own simulation outcomes.
use super::helpers::*;
use crate::*;

#[test]
fn animation_data_uses_versioned_json_payload_shape() {
    let program = seeded_program("mine();");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.mining_speed = 4;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[OreAnimationData {
        ore_id: 1,
        max_amount: 8,
    }]);

    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");
    assert_eq!(payload["v"], 2);
    assert_eq!(payload["robots"]["robot"][0]["robotnr"], 0);
    assert_eq!(payload["robots"]["robot"][0]["locations"][0]["l"], 1);
    assert!(
        payload["robots"]["robot"][0]["locations"][1]["cpu"]
            .as_array()
            .is_some_and(|cpu| !cpu.is_empty()),
        "mining cycle should record CPU micro-steps: {data}"
    );
    assert_eq!(payload["robots"]["robot"][0]["locations"][1]["A"], 4);
    assert_eq!(payload["robots"]["robot"][0]["locations"][1]["a"], 6);
    assert_eq!(
        animation_location_highlight_line(&payload["robots"]["robot"][0]["locations"][1]),
        Some(1)
    );
    assert_eq!(payload["ground"]["sizeX"], 4);
    assert_eq!(payload["ground"]["sizeY"], 4);
    assert_eq!(payload["ground"]["positions"][0]["x"], 0);
    assert_eq!(payload["ground"]["positions"][0]["c"][0]["A"], 8);
    assert_eq!(payload["ground"]["positions"][0]["c"][1]["t"], 1);
    assert_eq!(payload["ground"]["positions"][0]["c"][1]["A"], 4);
    assert_eq!(payload["oreTypes"]["A"]["id"], 1);
    assert_eq!(payload["oreTypes"]["A"]["max"], 8);
    assert!(!data.contains('<'));
    assert!(payload["robots"]["robot"][0].get("depotMaxA").is_none());
}

#[test]
fn animation_data_includes_depot_when_capacity_is_unlocked() {
    let program = seeded_program("mine(); dump(0);");
    let mut ground = Ground::new(4, 4);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;
    spec.mining_speed = 5;

    let mut capacity = [0; MAX_ORE_TYPES];
    capacity[0] = 10;
    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program).with_depot_capacity(capacity)],
    );
    let data = simulation.run_with_animation(&[OreAnimationData {
        ore_id: 1,
        max_amount: 8,
    }]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");

    assert_eq!(payload["robots"]["robot"][0]["depotMaxA"], 10);
    assert_eq!(payload["robots"]["robot"][0]["depotMaxB"], 0);
    assert_eq!(payload["robots"]["robot"][0]["depotMaxC"], 0);
    assert_eq!(payload["robots"]["robot"][0]["homeX"], 0);
    assert_eq!(payload["robots"]["robot"][0]["homeY"], 0);
    assert_eq!(payload["robots"]["robot"][0]["homeSize"], 1);
    // After mine then dump at spawn, depot should hold mined ore.
    let locations = payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("locations");
    let last = locations.last().expect("last location");
    assert_eq!(last["DA"], 4);
    assert_eq!(simulation.robot(0).depot()[0], 4);
}

#[test]
fn animation_data_depot_home_square_uses_ceil_robot_size_at_spawn_corner() {
    let ground = Ground::new(8, 8);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.robot_size = 1.5;

    let mut capacity = [0; MAX_ORE_TYPES];
    capacity[0] = 5;
    let robots = (0..4)
        .map(|_| {
            ScriptedRobot::new(spec.clone(), vec![RobotAction::Wait]).with_depot_capacity(capacity)
        })
        .collect();
    let mut simulation = Simulation::new(ground, 1, robots);
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");

    let robots = payload["robots"]["robot"].as_array().expect("robots");
    assert_eq!(robots[0]["homeX"], 0);
    assert_eq!(robots[0]["homeY"], 0);
    assert_eq!(robots[0]["homeSize"], 2);
    assert_eq!(robots[1]["homeX"], 0);
    assert_eq!(robots[1]["homeY"], 6);
    assert_eq!(robots[1]["homeSize"], 2);
    assert_eq!(robots[2]["homeX"], 6);
    assert_eq!(robots[2]["homeY"], 0);
    assert_eq!(robots[2]["homeSize"], 2);
    assert_eq!(robots[3]["homeX"], 6);
    assert_eq!(robots[3]["homeY"], 6);
    assert_eq!(robots[3]["homeSize"], 2);
}

#[test]
fn animation_data_records_wait_action_index_on_idle_cycles() {
    let ground = Ground::new(4, 4);
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Wait, RobotAction::Wait],
        )],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":1"#),
        "wait cycles should emit action index 1: {data}"
    );
}

#[test]
fn animation_data_records_scan_action_index_while_scanning() {
    let source = "scan(); if (oreDistance() < 0) { rotate(0); } mine();";
    let program = seeded_program(source);

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        data.contains(r#""a":0"#),
        "scan-busy wait cycles should emit action index 0: {data}"
    );
    let scan_marks = data.matches(r#""a":0"#).count();
    assert!(
        scan_marks >= 2,
        "expected multiple scan-busy cycles, found {scan_marks} in {data}"
    );
}

#[test]
fn animation_data_records_stuck_status_strings() {
    struct Case {
        name: &'static str,
        expected_status: &'static str,
        data: fn() -> String,
    }

    let cases = [
        Case {
            name: "wait",
            expected_status: "wait",
            data: wait_status_animation_data,
        },
        Case {
            name: "scan",
            expected_status: "scan",
            data: scan_status_animation_data,
        },
        Case {
            name: "zero",
            expected_status: "zero",
            data: zero_status_animation_data,
        },
        Case {
            name: "cpu",
            expected_status: "cpu",
            data: cpu_status_animation_data,
        },
        Case {
            name: "battery",
            expected_status: "battery",
            data: battery_status_animation_data,
        },
        Case {
            name: "motion",
            expected_status: "motion",
            data: no_chunk_status_animation_data,
        },
        Case {
            name: "wall",
            expected_status: "wall",
            data: wall_status_animation_data,
        },
        Case {
            name: "robot",
            expected_status: "robot",
            data: robot_status_animation_data,
        },
    ];

    for case in cases {
        let data = (case.data)();
        assert!(
            data.contains(&format!(r#""s":"{}""#, case.expected_status)),
            "{} should emit stuck status {}: {data}",
            case.name,
            case.expected_status
        );
    }
}

fn wait_status_animation_data() -> String {
    let ground = Ground::new(4, 4);
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Wait, RobotAction::Wait],
        )],
    );
    simulation.run_with_animation(&[])
}

fn scan_status_animation_data() -> String {
    let source = "scan(); if (oreDistance() < 0) { rotate(0); } mine();";
    let program = seeded_program(source);

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 2, 2);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 1;
    spec.scan_time = 6;
    spec.scan_distance = 50;
    spec.max_turns = 4;

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

fn zero_status_animation_data() -> String {
    let program = seeded_program("move(0); rotate(0);");
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 3;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        3,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

fn cpu_status_animation_data() -> String {
    let program = seeded_program("int x = 1; int y = 2; mine();");
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut spec = RobotSpec::test_robot();
    spec.cpu_speed = 2;
    spec.mining_speed = 8;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

fn battery_status_animation_data() -> String {
    let ground = Ground::new(4, 4);
    let mut short = RobotSpec::test_robot();
    short.max_turns = 1;
    let mut long = RobotSpec::test_robot();
    long.max_turns = 3;

    let mut simulation = Simulation::new(
        ground,
        3,
        vec![
            ScriptedRobot::new(short, vec![RobotAction::Wait; 3]),
            ScriptedRobot::new(long, vec![RobotAction::Wait; 3]),
        ],
    );
    simulation.run_with_animation(&[])
}

fn no_chunk_status_animation_data() -> String {
    let program = seeded_program("move(1);");
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 0.0;
    spec.max_turns = 2;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        2,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.run_with_animation(&[])
}

fn wall_status_animation_data() -> String {
    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.backward_speed = 1.0;

    let mut simulation = Simulation::new(
        Ground::new(4, 4),
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Backward])],
    );
    simulation.run_with_animation(&[])
}

fn robot_status_animation_data() -> String {
    let mut mover = RobotSpec::test_robot();
    mover.forward_speed = 1.0;
    mover.robot_size = 1.0;
    mover.max_turns = 1;

    let mut blocker = RobotSpec::test_robot();
    blocker.robot_size = 1.0;
    blocker.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(2, 2),
        1,
        vec![
            ScriptedRobot::new(mover, vec![RobotAction::Forward]),
            ScriptedRobot::new(blocker, vec![RobotAction::Wait]),
        ],
    );
    simulation.run_with_animation(&[])
}

#[test]
fn animation_data_omits_wall_status_for_partial_wall_clip() {
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 10.0;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(5, 5),
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Forward])],
    );
    let data = simulation.run_with_animation(&[]);

    assert!(
        !data.contains(r#""s":"wall""#),
        "partial wall clip must not be labeled stuck: {data}"
    );
}

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
