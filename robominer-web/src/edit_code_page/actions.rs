//! Program source create/update/delete mutations for the edit code page.

use crate::{Request, is_post, mutation_i64, query_signed_i64};

use super::{
    EditCodePageState, default_edit_code_program_source, edit_code_program_source_from_state,
    format_save_with_optional_apply_message, program_source_write_page_error,
    selected_edit_code_source,
};

pub(super) struct EditCodeMutationOutcome {
    pub(super) message: Option<String>,
    pub(super) next_program_source_id: Option<i64>,
}

pub(super) async fn apply_edit_code_mutations(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<EditCodeMutationOutcome, crate::page_context::PageLoadError> {
    let mut message = None;
    let mut next_program_source_id = query_signed_i64(request, "nextProgramSourceId");
    let program_source_id = if is_post(request) {
        mutation_i64(request, "programSourceId").unwrap_or(0)
    } else {
        crate::query_i64(request, "programSourceId").unwrap_or(0)
    };

    if is_post(request) {
        match request.form.get("requestType").map(String::as_str) {
            Some("erase") if program_source_id > 0 => {
                if let Err(rejection) =
                    robominer_db::delete_program_source_for_user(pool, user_id, program_source_id)
                        .await?
                        .into_result()
                {
                    message = Some(format!(
                        "Unable to delete program: {}",
                        robominer_domain::rejection_messages::program_source_write_rejection_player_message(rejection)
                    ));
                } else {
                    next_program_source_id = None;
                    message = Some("Program deleted.".to_string());
                }
            }
            Some("update") => {
                let source_name = request.form.get("sourceName").cloned().unwrap_or_default();
                let source_code = request.form.get("sourceCode").cloned().unwrap_or_default();
                if program_source_id > 0 {
                    if let Err(rejection) = robominer_domain::update_program_source(
                        pool,
                        robominer_db::ProgramSourceWriteRequest {
                            user_id,
                            program_source_id,
                            source_name,
                            source_code,
                        },
                    )
                    .await
                    .map_err(program_source_write_page_error)?
                    .into_result()
                    {
                        message = Some(format!(
                            "Unable to save program: {}",
                            robominer_domain::rejection_messages::program_source_write_rejection_player_message(rejection)
                        ));
                    } else {
                        let applied = robominer_db::apply_verified_program_source_to_idle_robots(
                            pool,
                            user_id,
                            program_source_id,
                        )
                        .await?;
                        message = Some(format_save_with_optional_apply_message(
                            "Program saved.",
                            &applied,
                        ));
                    }
                } else if !source_name.is_empty() || !source_code.is_empty() {
                    match robominer_domain::create_program_source(
                        pool,
                        robominer_db::CreateProgramSourceRequest {
                            user_id,
                            source_name,
                            source_code,
                        },
                    )
                    .await
                    .map_err(program_source_write_page_error)?
                    .into_result()
                    {
                        Ok(created) => {
                            next_program_source_id = Some(created.program_source_id);
                            let applied =
                                robominer_db::apply_verified_program_source_to_idle_robots(
                                    pool,
                                    user_id,
                                    created.program_source_id,
                                )
                                .await?;
                            message = Some(format_save_with_optional_apply_message(
                                "Program created.",
                                &applied,
                            ));
                        }
                        Err(rejection) => {
                            message = Some(format!(
                                "Unable to save program: {}",
                                robominer_domain::rejection_messages::program_source_write_rejection_player_message(rejection)
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(EditCodeMutationOutcome {
        message,
        next_program_source_id,
    })
}

pub(super) async fn load_edit_code_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<EditCodePageState, crate::page_context::PageLoadError> {
    let mutation = apply_edit_code_mutations(pool, user_id, request).await?;
    let prefer_stored_selection = !is_post(request)
        && !mutation
            .next_program_source_id
            .is_some_and(|source_id| source_id > 0);

    let program_sources = robominer_db::list_program_source_states_for_user(pool, user_id).await?;
    let selected_source =
        selected_edit_code_source(&program_sources, mutation.next_program_source_id);

    let selected_program_source = selected_source
        .map(edit_code_program_source_from_state)
        .unwrap_or_else(default_edit_code_program_source);
    let selected_program_source_id = selected_source.map(|state| state.source.id).unwrap_or(-1);

    Ok(EditCodePageState {
        selected_program_source_id,
        selected_program_source,
        program_sources,
        message: mutation.message,
        prefer_stored_selection,
    })
}
