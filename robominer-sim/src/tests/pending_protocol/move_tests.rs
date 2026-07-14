use super::{protocol_simulation, runtime_move_program};
use super::super::helpers::*;
use crate::action_mapping::PendingExpressionAction;
use crate::physics::ActionResult;
use crate::*;

#[test]
fn partial_move_chunk_clears_action_result_and_keeps_sim_pending() {
    let mut simulation = protocol_simulation(
        "if (move(2.0) > 1.9) { rotate(90); } else { mine(); }",
        5,
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert!(
        simulation.pending_expression_action(0).is_some(),
        "sim should keep chunking state after the first move cycle"
    );
    assert_eq!(
        simulation.test_action_result(0),
        None,
        "partial chunks must not expose a finished action result to the runner"
    );
    assert!(
        simulation.program_runner(0).unwrap().has_pending_physical(),
        "runner should still hold the logical move while sim chunks"
    );
    assert!(
        simulation.test_action_result_expected(0),
        "sim should still expect a finished move result for the runner"
    );
}

#[test]
fn sim_pending_chunks_skip_runner_until_move_finishes() {
    let mut simulation = protocol_simulation(
        "if (move(2.0) > 1.9) { rotate(0); } else { mine(); }",
        5,
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    let runner = simulation.program_runner(0).unwrap();
    assert!(runner.has_pending_physical());

    simulation.advance_test_turn();

    assert!(
        simulation.pending_expression_action(0).is_none(),
        "sim pending should clear after the final move chunk"
    );
    assert_close(simulation.test_action_result(0).unwrap(), 2.0);
    assert!(
        simulation.program_runner(0).unwrap().has_pending_physical(),
        "runner should still resume the expression after sim chunking completes"
    );

    simulation.advance_test_turn();

    assert!(
        !simulation
            .program_runner(0)
            .unwrap()
            .has_pending_physical(),
        "runner should finish the logical move once the accumulated result is consumed"
    );
}

#[test]
fn record_action_result_accumulates_partial_chunks_then_finishes() {
    let mut simulation = protocol_simulation("if (move(2.0) > 0) { rotate(90); }", 3);
    simulation.prepare_test_run();
    simulation.test_set_pending_expression(
        0,
        Some(PendingExpressionAction::Move {
            remaining: 2.0,
            accumulated: 0.0,
        }),
    );

    simulation.test_record_action_result(0, ActionResult::Value(1.0));
    assert_eq!(simulation.test_action_result(0), None);
    assert!(simulation.pending_expression_action(0).is_some());

    simulation.test_record_action_result(0, ActionResult::Value(1.0));
    assert_close(simulation.test_action_result(0).unwrap(), 2.0);
    assert!(simulation.pending_expression_action(0).is_none());
}

#[test]
fn blocked_move_chunk_finishes_pending_with_accumulated_travel() {
    let mut simulation = protocol_simulation("if (move(2.0) > 0) { rotate(90); }", 3);
    simulation.prepare_test_run();
    simulation.test_set_pending_expression(
        0,
        Some(PendingExpressionAction::Move {
            remaining: 2.0,
            accumulated: 0.0,
        }),
    );

    simulation.test_record_action_result(0, ActionResult::Value(0.0));

    assert_close(simulation.test_action_result(0).unwrap(), 0.0);
    assert!(simulation.pending_expression_action(0).is_none());
}

#[test]
fn multi_chunk_rotate_expression_uses_sim_pending_without_runner_resume() {
    let source = "if (rotate(180) == 180) { mine(); } else { move(1); }";
    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 1.0;
    spec.rotate_speed = 90;
    spec.mining_speed = 4;
    spec.max_turns = 4;

    let mut ground = Ground::new(5, 5);
    ground.at_mut(0, 0).add_ore(0, 8);

    let mut simulation = Simulation::new(
        ground,
        4,
        vec![ScriptedRobot::from_executable_program(
            spec,
            &runtime_move_program(source),
        )],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert!(
        simulation.pending_expression_action(0).is_some(),
        "first rotate chunk should leave sim pending state"
    );
    assert_eq!(simulation.test_action_result(0), None);
    assert!(
        simulation.program_runner(0).unwrap().has_pending_physical()
    );

    simulation.advance_test_turn();

    assert!(simulation.pending_expression_action(0).is_none());
    assert_close(simulation.test_action_result(0).unwrap(), 180.0);

    simulation.advance_test_turn();

    assert_eq!(simulation.robot(0).ore_at(0), 4);
    assert_eq!(simulation.robot(0).actions_done()[6], 1);
}

#[test]
fn dynamic_move_program_uses_robot_speed_from_execution_context() {
    let program = robominer_program::compile_executable_source(
        "if (move(robot.forwardSpeed) < 1) { rotate(150); } else { rotate(0); }",
    )
    .expect("dynamic move program should compile");
    assert!(program.requires_runtime());

    let mut spec = RobotSpec::test_robot();
    spec.forward_speed = 2.0;
    spec.rotate_speed = 90;
    spec.cpu_speed = 72;
    spec.max_turns = 1;

    let mut simulation = Simulation::new(
        Ground::new(10, 10),
        1,
        vec![ScriptedRobot::from_executable_program(spec, &program)],
    );
    simulation.prepare_test_run();
    simulation.advance_test_turn();

    assert_eq!(simulation.robot(0).actions_done()[2], 1);
    assert_eq!(simulation.robot(0).position().orientation, 45);
}
