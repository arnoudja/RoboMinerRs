use super::super::helpers::*;
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
    assert_eq!(payload["robots"]["robot"][0]["cpuspeed"], 72);
    assert!(
        payload["robots"]["robot"][0]["locations"][1]["cpu"]
            .as_array()
            .is_some_and(|cpu| !cpu.is_empty()),
        "turn should record CPU micro-steps: {data}"
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
fn animation_data_records_typed_cpu_step_return_values() {
    let program = seeded_program("int x = 3 + 4;\nbool y = true;\ndouble z = 1.5;");
    let ground = Ground::new(4, 4);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");

    let mut kinds = Vec::new();
    let mut values = Vec::new();
    for location in payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations")
    {
        if let Some(cpu) = location.get("cpu").and_then(|value| value.as_array()) {
            for step in cpu {
                if let Some(result) = step.get("r") {
                    kinds.push(
                        result
                            .get("k")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string(),
                    );
                    values.push(
                        result
                            .get("v")
                            .and_then(|value| value.as_f64())
                            .unwrap_or(0.0),
                    );
                }
            }
        }
    }

    assert!(
        kinds.iter().any(|kind| kind == "i")
            && values.iter().any(|value| (*value - 7.0).abs() < 1e-9),
        "should record int 3+4 result: {data}"
    );
    assert!(
        kinds.iter().any(|kind| kind == "b")
            && values
                .iter()
                .zip(kinds.iter())
                .any(|(value, kind)| kind == "b" && (*value - 1.0).abs() < 1e-9),
        "should record bool true: {data}"
    );
    assert!(
        kinds.iter().any(|kind| kind == "f")
            && values
                .iter()
                .zip(kinds.iter())
                .any(|(value, kind)| kind == "f" && (*value - 1.5).abs() < 1e-9),
        "should record float 1.5: {data}"
    );
}

#[test]
fn animation_data_records_visible_program_variables() {
    let program = seeded_program("int x = 1;\nbool y = true;\ndouble z = 1.5;");
    let ground = Ground::new(4, 4);

    let mut spec = RobotSpec::test_robot();
    spec.max_turns = 1;
    spec.cpu_speed = 72;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    let data = simulation.run_with_animation(&[]);
    let payload: serde_json::Value =
        serde_json::from_str(&data).expect("animation payload should be JSON");

    let mut saw_all_three = false;
    for location in payload["robots"]["robot"][0]["locations"]
        .as_array()
        .expect("robot locations")
    {
        if let Some(cpu) = location.get("cpu").and_then(|value| value.as_array()) {
            for step in cpu {
                let Some(vs) = step.get("vs") else {
                    continue;
                };
                let x = vs
                    .get("x")
                    .and_then(|v| v.get("v"))
                    .and_then(|v| v.as_f64());
                let y = vs.get("y");
                let z = vs.get("z");
                if x == Some(1.0)
                    && y.and_then(|v| v.get("k")).and_then(|v| v.as_str()) == Some("b")
                    && y.and_then(|v| v.get("v")).and_then(|v| v.as_f64()) == Some(1.0)
                    && z.and_then(|v| v.get("k")).and_then(|v| v.as_str()) == Some("f")
                    && z.and_then(|v| v.get("v"))
                        .and_then(|v| v.as_f64())
                        .is_some_and(|value| (value - 1.5).abs() < 1e-9)
                {
                    assert_eq!(
                        vs.get("x")
                            .and_then(|v| v.get("k"))
                            .and_then(|v| v.as_str()),
                        Some("i")
                    );
                    saw_all_three = true;
                    break;
                }
            }
        }
    }

    assert!(
        saw_all_three,
        "should record typed locals x/y/z in cpu[].vs: {data}"
    );
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
