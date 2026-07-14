use crate::*;

use super::helpers::*;

#[test]
fn compatibility_fixtures_match_verification_expectations() {
    for fixture in compatibility_fixtures() {
        let result = verify_source(fixture.source);

        if let Some(error_contains) = fixture.expected_error_contains {
            assert!(!result.verified, "{} unexpectedly verified", fixture.name);
            assert_eq!(result.compiled_size, -1, "{}", fixture.name);
            assert!(
                result.error_description.contains(error_contains),
                "{}: expected error containing {error_contains:?}, got {:?}",
                fixture.name,
                result.error_description
            );
        } else {
            assert!(
                result.verified,
                "{}: {}",
                fixture.name, result.error_description
            );
            if let Some(expected_size) = fixture.expected_size {
                assert_eq!(result.compiled_size, expected_size, "{}", fixture.name);
            } else {
                assert!(result.compiled_size > 0, "{}", fixture.name);
            }
            assert_eq!(result.error_description, "", "{}", fixture.name);
        }
    }
}

#[test]
fn verify_source_requires_executable_program_support() {
    assert_valid_any_size("do { mine(); } while (false);");
    assert_valid_any_size("ore(0);");
    assert_valid_any_size("time();");
    assert_valid_any_size("scan();");

    let source = "int rot = 0; while (true) { while (mine()) { rot = 100; } }";
    let verification = verify_source(source);
    assert!(verification.verified, "{}", verification.error_description);
    compile_executable_source(source).expect("verified source should compile for execution");
}

#[test]
fn verifies_minus_equals_assignment() {
    assert_valid_any_size("int direction = 90; direction -= 30; direction -= direction;");
}

#[test]
fn verifies_braceless_if_body_followed_by_statement() {
    assert_valid_any_size(
        "int rot = 315 - robot.orientation;\nif (rot > 180)\n    rot -= 360;\n\nrotate(rot);",
    );
    assert_valid_any_size("if (true) move(1); rotate(90);");
    assert_valid_any_size("int rot = 0; if (true) rot -= 1; else rot += 1; mine();");
}

#[test]
fn verifies_plus_equals_assignment() {
    assert_valid_any_size("int direction = 0; direction += 30; direction += direction;");
    assert_valid_any_size(
        "int direction = 0; scan(direction); while (oreType() != 1) { direction += 30; scan(direction); }",
    );
}

#[test]
fn verifies_not_equal_operator() {
    assert_valid_any_size("while (oreType() != 1) { scan(30); }");
    assert_valid_any_size("if (mine() != 0) { dump(1); }");
}

#[test]
fn robot_property_program_verifies() {
    assert_valid_any_size("move(robot.forwardSpeed);");
    assert_valid_any_size("if (robot.orientation == 135) { move(1); }");
    assert_valid_any_size("if (robot.xPos == 0 && robot.yPos == 0) { move(1); }");
}

#[test]
fn dynamic_move_in_assignment_compiles() {
    assert_valid_any_size("float d = move(robot.forwardSpeed);");
}

#[test]
fn compiles_literal_action_sequence_for_simulation() {
    let program = compile_executable_source("move(1.5); rotate(-45); mine(); dump(0);")
        .expect("source should compile to executable actions");

    assert_eq!(
        program.actions(),
        &[
            ExecutableAction::Move(1.5),
            ExecutableAction::Rotate(-45.0),
            ExecutableAction::Mine,
            ExecutableAction::Dump(0),
        ]
    );
}

#[test]
fn compiles_time_and_ore_control_flow_for_simulation() {
    let program = compile_executable_source(
        "if (time() > 0 && ore(0) == 0) { move(1); } else { rotate(90); } while (time() > 1) { mine(); }",
    )
    .expect("source should compile to executable control flow");

    let mut runner = program.runner();
    let mut context = test_context(3, None);

    assert_eq!(
        runner.next_action(&mut context),
        Some(ExecutableAction::Move(1.0))
    );
    let mut after_move = test_context(3, Some(1.0));
    assert_eq!(
        runner.next_action(&mut after_move),
        Some(ExecutableAction::Mine)
    );
}

#[test]
fn executable_while_semicolon_matches_empty_block() {
    for source in ["while (mine());", "while (mine()) {}"] {
        let program = compile_executable_source(source)
            .unwrap_or_else(|err| panic!("{source} should compile: {err}"));
        let mut runner = program.runner();

        let mut first_context = test_context(3, None);
        assert_eq!(
            runner.next_action(&mut first_context),
            Some(ExecutableAction::Mine),
            "{source}"
        );

        let mut mined_context = test_context(3, Some(4.0));
        assert_eq!(
            runner.next_action(&mut mined_context),
            Some(ExecutableAction::Mine),
            "{source}"
        );

        let mut depleted_context = test_context(3, Some(0.0));
        assert_eq!(runner.next_action(&mut depleted_context), None, "{source}");
    }
}
