use crate::{
    Request, Response, ServerConfig, is_post, login_redirect, query_i64, query_signed_i64,
    session_username,
};

#[derive(Debug)]
pub(super) struct EditCodePageState {
    pub(super) selected_program_source_id: i64,
    pub(super) selected_program_source: EditCodeProgramSource,
    pub(super) program_sources: Vec<robominer_db::ProgramSourceStateRecord>,
    pub(super) message: Option<String>,
    pub(super) claimed_results: robominer_db::ClaimedUserResults,
}

#[derive(Debug, Clone)]
pub(super) struct EditCodeProgramSource {
    pub(super) source_name: String,
    pub(super) source_code: String,
    pub(super) compiled_size: i32,
    pub(super) error_description: String,
    pub(super) linked_robot_count: i64,
    pub(super) verified: bool,
}

pub(super) async fn edit_code_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    if let Some(response) = crate::csrf::reject_invalid_csrf(request, user_id) {
        return response;
    }
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Edit code requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let result = load_edit_code_page_state(pool, user_id, request).await;

    match result {
        Ok(state) => crate::csrf::html_with_csrf(
            request,
            user_id,
            render::render_edit_code_page(
                session_username(request),
                crate::app_shell::hud_markup(request, config)
                    .await
                    .as_deref(),
                &state,
            ),
        ),
        Err(error) => Response::service_unavailable(format!("Unable to load edit code: {error}")),
    }
}

async fn load_edit_code_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<EditCodePageState, robominer_domain::DomainError> {
    let claim_result = robominer_db::claim_user_results(pool, user_id).await?;

    let mut message = None;
    let mut next_program_source_id = query_signed_i64(request, "nextProgramSourceId");
    let program_source_id = query_i64(request, "programSourceId").unwrap_or(0);

    if is_post(request) {
        match request.form.get("requestType").map(String::as_str) {
            Some("erase") if program_source_id > 0 => {
                if let Err(rejection) =
                    robominer_db::delete_program_source_for_user(pool, user_id, program_source_id)
                        .await?
                {
                    message = Some(format!(
                        "Unable to delete program: {}",
                        program_source_write_rejection_message(rejection)
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
                    .await?
                    {
                        message = Some(format!(
                            "Unable to save program: {}",
                            program_source_write_rejection_message(rejection)
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
                    .await?
                    {
                        Ok(created) => {
                            if next_program_source_id.is_none_or(|source_id| source_id <= 0) {
                                next_program_source_id = Some(created.program_source_id);
                            }
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
                                program_source_write_rejection_message(rejection)
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let program_sources = robominer_db::list_program_source_states_for_user(pool, user_id).await?;
    let selected_source = selected_edit_code_source(&program_sources, next_program_source_id);

    let selected_program_source = selected_source
        .map(edit_code_program_source_from_state)
        .unwrap_or_else(default_edit_code_program_source);
    let selected_program_source_id = selected_source.map(|state| state.source.id).unwrap_or(-1);

    Ok(EditCodePageState {
        selected_program_source_id,
        selected_program_source,
        program_sources,
        message,
        claimed_results: claim_result,
    })
}

pub(super) fn selected_edit_code_source(
    program_sources: &[robominer_db::ProgramSourceStateRecord],
    requested_program_source_id: Option<i64>,
) -> Option<&robominer_db::ProgramSourceStateRecord> {
    match requested_program_source_id {
        Some(source_id) if source_id > 0 => program_sources
            .iter()
            .find(|state| state.source.id == source_id),
        Some(_) => None,
        None => program_sources.first(),
    }
}

pub(super) fn edit_code_save_block_reason(
    source_name: &str,
    source_code: &str,
) -> Option<&'static str> {
    if source_name.trim().is_empty() {
        return Some(
            robominer_domain::program_source_write_rejection_player_message(
                robominer_db::ProgramSourceWriteRejection::EmptySourceName,
            ),
        );
    }
    if source_code.trim().is_empty() {
        return Some(
            robominer_domain::program_source_write_rejection_player_message(
                robominer_db::ProgramSourceWriteRejection::EmptySourceCode,
            ),
        );
    }
    None
}

fn format_program_source_apply_message(applied: &robominer_db::AppliedProgramSource) -> String {
    robominer_domain::format_program_source_apply_player_message(applied)
}

/// Combine a save/create banner with linked-robot apply results when anything was updated.
pub(super) fn format_save_with_optional_apply_message(
    saved_label: &str,
    applied: &robominer_db::AppliedProgramSource,
) -> String {
    if applied.applied_robots == 0 && applied.warnings.is_empty() {
        saved_label.to_string()
    } else {
        format!(
            "{saved_label} {}",
            format_program_source_apply_message(applied)
        )
    }
}

pub(super) fn edit_code_program_source_from_state(
    state: &robominer_db::ProgramSourceStateRecord,
) -> EditCodeProgramSource {
    EditCodeProgramSource {
        source_name: state.source.source_name.clone(),
        source_code: state.source.source_code.clone().unwrap_or_default(),
        compiled_size: state.source.compiled_size,
        error_description: state.source.error_description.clone(),
        linked_robot_count: state.linked_robot_count,
        verified: state.source.verified,
    }
}

pub(super) fn default_edit_code_program_source() -> EditCodeProgramSource {
    EditCodeProgramSource {
        source_name: String::new(),
        source_code: "move(1);\nmine();".to_string(),
        compiled_size: 4,
        error_description: String::new(),
        linked_robot_count: 0,
        verified: false,
    }
}

pub(super) fn program_source_write_rejection_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
) -> &'static str {
    robominer_domain::program_source_write_rejection_player_message(rejection)
}

mod editor;
mod library;
mod render;
mod scripts;

#[cfg(test)]
mod tests;
