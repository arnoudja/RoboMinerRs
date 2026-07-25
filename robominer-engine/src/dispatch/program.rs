use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::ensure_positive_user_id;
use crate::cli::ProgramCommand;
use crate::database::connect_database;
use crate::program::{
    create_program_source, delete_program_source, program_source_states, update_program_source,
};
use crate::verify::{
    SimulateSourceOptions, simulate_source_file, verify as verify_program, verify_source_file,
};

pub(crate) async fn dispatch_program(
    database_url: Option<String>,
    config: Option<PathBuf>,
    command: ProgramCommand,
) -> Result<()> {
    match command {
        ProgramCommand::Verify { program_source_id } => {
            let pool = connect_database(database_url, config).await?;
            verify_program(&pool, program_source_id).await
        }
        ProgramCommand::VerifySource { source_file } => verify_source_file(&source_file),
        ProgramCommand::SimulateSource {
            source_file,
            robot,
            turns,
            size_x,
            size_y,
            ore_x,
            ore_y,
            ore_type,
            ore_amount,
            mining_speed,
            forward_speed,
            backward_speed,
            rotate_speed,
        } => simulate_source_file(SimulateSourceOptions {
            source_file,
            robot_files: robot,
            turns,
            size_x,
            size_y,
            ore_x,
            ore_y,
            ore_type,
            ore_amount,
            mining_speed,
            forward_speed,
            backward_speed,
            rotate_speed,
        }),
        ProgramCommand::CreateSource {
            user_id,
            source_name,
            source_code,
        } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            create_program_source(
                &pool,
                robominer_db::CreateProgramSourceRequest {
                    user_id,
                    source_name,
                    source_code,
                },
            )
            .await
        }
        ProgramCommand::UpdateSource {
            user_id,
            program_source_id,
            source_name,
            source_code,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            update_program_source(
                &pool,
                robominer_db::ProgramSourceWriteRequest {
                    user_id,
                    program_source_id,
                    source_name,
                    source_code,
                },
            )
            .await
        }
        ProgramCommand::DeleteSource {
            user_id,
            program_source_id,
        } => {
            ensure_positive_user_id(user_id)?;
            ensure!(
                program_source_id > 0,
                "--program-source-id must be greater than zero"
            );
            let pool = connect_database(database_url, config).await?;
            delete_program_source(&pool, user_id, program_source_id).await
        }
        ProgramCommand::SourceStates { user_id } => {
            ensure_positive_user_id(user_id)?;
            let pool = connect_database(database_url, config).await?;
            program_source_states(&pool, user_id).await
        }
    }
}
