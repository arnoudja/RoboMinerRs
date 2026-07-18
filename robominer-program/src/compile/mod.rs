use crate::types::{CompileError, ExecutableProgram, Verification};

mod cache;
mod executable;
mod fixtures;
mod input;
mod program_size;

use executable::parse_executable_program;
use program_size::program_instruction_size;

pub use cache::{clear_compile_cache, compile_cache_stats};
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
    Ok(compile_executable_source_with_size(source)?.0)
}

pub fn compile_executable_source(source: &str) -> Result<ExecutableProgram, CompileError> {
    Ok(compile_executable_source_with_size(source)?.1)
}

fn compile_executable_source_with_size(
    source: &str,
) -> Result<(usize, ExecutableProgram), CompileError> {
    cache::get_or_insert_with(source, || {
        let program = parse_executable_program(source)?;
        Ok((program_instruction_size(&program), program))
    })
}
