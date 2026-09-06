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

#[test]
fn compiles_calls_and_forward_reference() {
    assert!(verify_source("move(f()); fn int f() { return 2; }").verified);
    assert!(verify_source("fn int add(int a, b) { return a + b; } move(add(1, 2));").verified);
}

#[test]
fn rejects_arity_mismatch_and_name_clash() {
    assert!(!verify_source("fn int f(a) { return a; } move(f());").verified);
    assert!(!verify_source("fn int f() { return 1; } int f; move(1);").verified);
    assert!(!verify_source("int f; fn int f() { return 1; } move(1);").verified);
}

#[test]
fn rejects_return_outside_function() {
    assert!(!verify_source("return 1;").verified);
}

#[test]
fn omitted_return_type_requires_agreement() {
    assert!(verify_source("fn f() { return 1; } move(f());").verified);
    assert!(!verify_source("fn f() { if (true) { return 1; } return 1.5; } move(f());").verified);
}
