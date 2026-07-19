use crate::{
    compatibility_fixture_source, compatibility_fixtures, compile_executable_source,
    compile_source, unparse_program,
};

#[test]
fn unparse_round_trip_preserves_compiled_size_for_fixtures() {
    for fixture in compatibility_fixtures()
        .iter()
        .filter(|fixture| fixture.expected_error_contains.is_none())
    {
        let Ok(program) = compile_executable_source(fixture.source) else {
            continue;
        };
        let source = unparse_program(&program);
        let size = compile_source(&source).unwrap_or_else(|error| {
            panic!(
                "unparsed fixture '{}' failed to compile: {error}\n---\n{source}\n---",
                fixture.name
            )
        });
        let original = compile_source(fixture.source).expect("original fixture size");
        assert_eq!(
            size, original,
            "size drift for fixture '{}'\n--- unparsed ---\n{source}",
            fixture.name
        );
    }
}

#[test]
fn unparse_round_trip_named_seed_programs() {
    for name in ["default_program", "seed_ai_1", "seed_ai_2", "flow_control"] {
        let source = compatibility_fixture_source(name);
        let program = compile_executable_source(source).expect("compile");
        let again = unparse_program(&program);
        let recompiled = compile_executable_source(&again).expect("recompile");
        assert_eq!(program.actions(), recompiled.actions());
        assert_eq!(program.requires_runtime(), recompiled.requires_runtime());
    }
}
