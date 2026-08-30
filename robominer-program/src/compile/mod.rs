use crate::types::{CompileError, ExecutableProgram};

mod cache;
mod executable;
mod fixtures;
mod input;
mod program_size;

use executable::parse_executable_program;
use program_size::program_instruction_size;
use std::time::{Duration, Instant};

pub use cache::{clear_compile_cache, compile_cache_stats};
pub use fixtures::{
    CompatibilityFixture, Verification, compatibility_fixture_source, compatibility_fixtures,
};

/// Matches `robominer_db::MAX_PROGRAM_SOURCE_CODE_BYTES` — defense in depth for
/// compile callers that do not go through DB validation.
pub const MAX_COMPILE_SOURCE_BYTES: usize = 16_384;

/// Soft wall-clock budget for a single compile; oversized/pathological sources
/// should already be rejected by [`MAX_COMPILE_SOURCE_BYTES`].
const MAX_COMPILE_DURATION: Duration = Duration::from_secs(2);

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
    if source.len() > MAX_COMPILE_SOURCE_BYTES {
        return Err(CompileError::new(format!(
            "Program source exceeds maximum length of {MAX_COMPILE_SOURCE_BYTES} bytes"
        )));
    }

    cache::get_or_insert_with(source, || {
        let started = Instant::now();
        let program = parse_executable_program(source)?;
        if started.elapsed() > MAX_COMPILE_DURATION {
            return Err(CompileError::new(
                "Program compile exceeded time budget".to_string(),
            ));
        }
        Ok((program_instruction_size(&program), program))
    })
}
