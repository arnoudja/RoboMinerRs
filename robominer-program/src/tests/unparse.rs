use crate::{
    compatibility_fixture_source, compatibility_fixtures, compile_executable_source,
    compile_source, unparse_program,
};

#[test]
fn unparse_emits_functions_before_main() {
    let source = "fn int f() { return 1; } move(f());";
    let program = compile_executable_source(source).expect("compile");
    let text = unparse_program(&program);
    let f_pos = text
        .find("fn int f")
        .or_else(|| text.find("int f"))
        .expect("function in unparse");
    let m_pos = text.find("move").expect("move in unparse");
    assert!(
        f_pos < m_pos,
        "functions must unparse before main statements: {text}"
    );
    compile_executable_source(&text).expect("unparsed program must recompile");
}

#[test]
fn program_size_includes_function_bodies() {
    let main_only = compile_source("move(1);").expect("main size");
    let with_function = compile_source("fn int f() { return 1; } move(f());").expect("fn size");
    assert!(
        with_function > main_only,
        "function body must increase program size: main={main_only} with_fn={with_function}"
    );
}

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

#[test]
fn unparse_preserves_decrement_and_unusual_dump_forms() {
    for source in [
        "int value = 3; --value; value--;",
        "dump(9);",
        "int slot = 1 + 2; dump(slot);",
        "dump(1 + 2);",
    ] {
        let program = compile_executable_source(source).unwrap_or_else(|error| {
            panic!("compile failed for {source:?}: {error}");
        });
        let unparsed = unparse_program(&program);
        let recompiled = compile_executable_source(&unparsed).unwrap_or_else(|error| {
            panic!("recompile failed for {source:?}\n---\n{unparsed}\n---\n{error}");
        });
        assert_eq!(
            program.actions(),
            recompiled.actions(),
            "action drift for {source:?}\n--- unparsed ---\n{unparsed}"
        );
        assert!(
            unparsed.contains("--") || unparsed.contains("dump"),
            "unexpected unparsed form for {source:?}: {unparsed}"
        );
    }
}
