use crate::*;

#[test]
fn user_arnoud_stats_scan_program_on_cerbonium_mini() {
    let source = "rotate(-45);\nscan(90);\nwhile (oreDistance() < 0) {\n    move(1);\n    scan(90);\n}\nrotate(90);\nmove(oreDistance());\nmine();";
    let program = robominer_program::compile_executable_source(source).expect("compile");

    let mut ground = Ground::new(10, 10);
    ground.add_ore_heap(4, 4, 0, 4, 4);

    let spec = RobotSpec {
        robot_id: 11,
        max_turns: 200,
        max_ore: 50,
        mining_speed: 5,
        cpu_speed: 20,
        forward_speed: 1.571,
        backward_speed: 0.714,
        rotate_speed: 952,
        robot_size: 1.38,
        scan_time: 6,
        scan_distance: 50,
    };

    let ore_ids = vec![1_i64];
    let mut sim = Simulation::new_with_ore_ids(
        ground,
        20,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
        ore_ids,
    );

    sim.run();

    let robot = sim.robot(0);
    assert!(
        robot.total_ore() > 0,
        "expected ore with scan_distance=50 and scan_time=6, got {} move actions",
        robot.actions_done()[2]
    );
}
