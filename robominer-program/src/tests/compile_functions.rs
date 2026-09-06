use crate::*;

#[test]
fn executable_program_exposes_functions_map() {
    let program = compile_executable_source("fn answer() { return 42; } move(answer());")
        .expect("function program should compile");
    assert!(
        program.functions.contains_key("answer"),
        "compiled program must expose function registry"
    );
}

#[test]
fn executable_program_functions_default_empty() {
    let program = compile_executable_source("move(1);").expect("move compiles");
    assert!(program.functions.is_empty());
}

#[test]
fn compiles_fn_forms_and_rejects_type_fn() {
    assert!(verify_source("fn f() { } move(1);").verified);
    assert!(verify_source("fn int f() { return 1; } move(1);").verified);
    assert!(verify_source("int f() { return 1; } move(1);").verified);
    let bad = verify_source("int fn f() { return 1; } move(1);");
    assert!(!bad.verified);
    assert!(
        bad.error_description.to_lowercase().contains("fn")
            || bad.error_description.to_lowercase().contains("syntax"),
        "{}",
        bad.error_description
    );
}

#[test]
fn rejects_nested_function_and_reserved_names() {
    assert!(!verify_source("fn outer() { fn inner() { } } move(1);").verified);
    assert!(!verify_source("fn move() { } move(1);").verified);
    assert!(!verify_source("fn while() { } move(1);").verified);
}
