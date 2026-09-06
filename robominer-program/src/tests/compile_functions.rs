use crate::*;

#[test]
#[ignore = "function parsing added in Task 2"]
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
