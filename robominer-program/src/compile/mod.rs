use crate::types::{CompileError, ExecutableProgram, Verification};

mod executable;
mod fixtures;
mod input;
mod program_size;

use executable::parse_executable_program;
use program_size::program_instruction_size;

pub use fixtures::{compatibility_fixture_source, compatibility_fixtures};

pub fn verify_source(source: &str) -> Verification {
    match compile_executable_source_with_size(source) {
        Ok((size, _)) => Verification {
            verified: true,
            compiled_size: size as i32,
            error_description: String::new(),
        },
        Err(error) => Verification {
            verified: false,
            compiled_size: -1,
            error_description: error.to_string(),
        },
    }
}

pub fn compile_source(source: &str) -> Result<usize, CompileError> {
    Ok(program_instruction_size(&parse_executable_program(source)?))
}

pub fn compile_executable_source(source: &str) -> Result<ExecutableProgram, CompileError> {
    parse_executable_program(source)
}

fn compile_executable_source_with_size(
    source: &str,
) -> Result<(usize, ExecutableProgram), CompileError> {
    let program = parse_executable_program(source)?;
    Ok((program_instruction_size(&program), program))
}
