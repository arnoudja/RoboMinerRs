use crate::output::escape_state_field;
use crate::verify::mark_program_source_verification;
use anyhow::{Context, Result, anyhow};

pub(crate) async fn create_program_source(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::CreateProgramSourceRequest,
) -> Result<()> {
    let source_code = request.source_code.clone();

    match robominer_domain::create_program_source(pool, request)
        .await
        .context("failed to create program source")?
    {
        Ok(result) => {
            mark_program_source_verification(pool, result.program_source_id, &source_code).await?;
            println!("Created program source {}", result.program_source_id);
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to create program source: {}",
            robominer_domain::program_source_write_rejection_cli_message(rejection)
        )),
    }
}

pub(crate) async fn update_program_source(
    pool: &robominer_db::MySqlPool,
    request: robominer_db::ProgramSourceWriteRequest,
) -> Result<()> {
    let user_id = request.user_id;
    let program_source_id = request.program_source_id;
    let source_code = request.source_code.clone();

    match robominer_domain::update_program_source(pool, request)
        .await
        .context("failed to update program source")?
    {
        Ok(()) => {
            mark_program_source_verification(pool, program_source_id, &source_code).await?;
            let applied = robominer_db::apply_verified_program_source_to_idle_robots(
                pool,
                user_id,
                program_source_id,
            )
            .await
            .context("failed to apply verified program source to robots")?;
            println!(
                "Updated program source {program_source_id}; applied to {} robot(s)",
                applied.applied_robots
            );
            print_program_source_warnings(&applied.warnings);
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to update program source: {}",
            robominer_domain::program_source_write_rejection_cli_message(rejection)
        )),
    }
}

pub(crate) async fn delete_program_source(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    program_source_id: i64,
) -> Result<()> {
    match robominer_domain::delete_program_source(pool, user_id, program_source_id)
        .await
        .context("failed to delete program source")?
    {
        Ok(()) => {
            println!("Deleted program source {program_source_id}");
            Ok(())
        }
        Err(rejection) => Err(anyhow!(
            "unable to delete program source: {}",
            robominer_domain::program_source_write_rejection_cli_message(rejection)
        )),
    }
}

fn print_program_source_warnings(warnings: &[robominer_db::ProgramSourceApplyWarning]) {
    for warning in warnings {
        println!(
            "WARNING Unable to apply the code to robot {}: {}",
            warning.robot_name,
            robominer_domain::program_source_apply_warning_message(warning.reason)
        );
    }
}

pub(crate) async fn program_source_states(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
) -> Result<()> {
    let states = robominer_domain::list_program_source_states(pool, user_id)
        .await
        .context("failed to load program source states")?;

    for state in states {
        println!(
            "S\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            state.source.id,
            escape_state_field(&state.source.source_name),
            escape_state_field(state.source.source_code.as_deref().unwrap_or_default()),
            state.source.verified,
            state.source.compiled_size,
            escape_state_field(&state.source.error_description),
            state.linked_robot_count,
            state.source.user_id
        );
    }

    Ok(())
}
