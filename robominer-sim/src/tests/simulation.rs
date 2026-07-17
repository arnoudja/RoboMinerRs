use super::helpers::*;
use crate::*;

#[test]
fn mines_ore_using_legacy_distribution_rules() {
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 10);
    ground.at_mut(0, 0).add_ore(1, 6);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 5;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        ground,
        1,
        vec![ScriptedRobot::new(spec, vec![RobotAction::Mine])],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).ore_at(0), 3);
    assert_eq!(simulation.robot(0).ore_at(1), 2);
    assert_eq!(simulation.robot(0).last_mined(), 5);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 7);
    assert_eq!(simulation.ground().at(0, 0).ore_at(1), 4);
}

#[test]
fn dump_all_returns_carried_ore_to_current_ground_unit() {
    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 10);
    ground.at_mut(0, 0).add_ore(1, 6);

    let mut spec = RobotSpec::test_robot();
    spec.mining_speed = 5;
    spec.max_turns = 2;

    let mut simulation = Simulation::new(
        ground,
        2,
        vec![ScriptedRobot::new(
            spec,
            vec![RobotAction::Mine, RobotAction::DumpAll],
        )],
    );

    simulation.run();

    assert_eq!(simulation.robot(0).total_ore(), 0);
    assert_eq!(simulation.ground().at(0, 0).ore_at(0), 10);
    assert_eq!(simulation.ground().at(0, 0).ore_at(1), 6);
}

#[test]
fn score_matches_legacy_ore_tiers() {
    let ore = ore_amounts(&[(0, 35), (1, 100), (2, 500)]);

    assert_close(calculate_score(ore), 999.99);
}

#[test]
fn ore_heap_matches_legacy_radial_shape() {
    let mut ground = Ground::new(5, 5);

    ground.add_ore_heap(2, 2, 0, 10, 2);

    assert_eq!(ground.at(2, 2).ore_at(0), 10);
    assert_eq!(ground.at(1, 2).ore_at(0), 5);
    assert_eq!(ground.at(2, 1).ore_at(0), 5);
    assert_eq!(ground.at(0, 2).ore_at(0), 0);
    assert_eq!(ground.at(1, 1).ore_at(0), 3);
}

#[test]
fn animation_data_uses_legacy_javascript_payload_shape() {
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

    assert!(data.contains("var myRobots = {robot: ["));
    assert!(data.contains("robotnr:0"));
    assert!(data.contains("locations:[{x:0.0,y:0.0,o:45,A:0,B:0,C:0}"));
    assert!(data.contains("{A:4,a:6}"));
    assert!(data.contains("var myGround = {sizeX:4,sizeY:4,positions:["));
    assert!(data.contains("{x:0,y:0,c:[{A:8},{t:1,A:4}]"));
    assert!(data.contains("var myOreTypes = {A:{id:1,max:8}};"));
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
        data.contains("a:1"),
        "wait cycles should emit action index 1: {data}"
    );
}
