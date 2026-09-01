use crate::{Request, Response, ServerConfig};

mod actions;
mod editor;
mod library;
mod render;
mod scripts;

#[cfg(test)]
mod tests;

use actions::load_edit_code_page_state;

#[derive(Debug)]
pub(super) struct EditCodePageState {
    pub(super) selected_program_source_id: i64,
    pub(super) selected_program_source: EditCodeProgramSource,
    pub(super) program_sources: Vec<robominer_db::ProgramSourceStateRecord>,
    pub(super) message: Option<String>,
    /// When true, the page script may restore the last client-stored program selection
    /// (GET without an explicit `nextProgramSourceId`). POST responses keep the server choice.
    pub(super) prefer_stored_selection: bool,
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

pub(super) async fn edit_code_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    let result = load_edit_code_page_state(session.pool, session.user_id, request).await;

    match result {
        Ok(state) => {
            session
                .html_with_hud(request, config, |username, hud| {
                    render::render_edit_code_page(username, hud, &state)
                })
                .await
        }
        Err(error) => crate::page_context::page_load_error("edit code", error),
    }
}

/// Map a `create_program_source` / `update_program_source` domain failure into a
/// page-load error. These façades are only expected to fail with
/// [`robominer_domain::DomainError::Database`]; any other variant is unreachable on
/// this path today and maps to a fixed configuration error (no domain Display leak).
pub(super) fn program_source_write_page_error(
    error: robominer_domain::DomainError,
) -> crate::page_context::PageLoadError {
    crate::page_context::PageLoadError::from_database(error).unwrap_or_else(|_| {
        crate::page_context::PageLoadError::from(sqlx::Error::Configuration(
            "unexpected domain error on program source write".into(),
        ))
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
            robominer_domain::rejection_messages::program_source_write_rejection_player_message(
                robominer_db::ProgramSourceWriteRejection::EmptySourceName,
            ),
        );
    }
    if source_code.trim().is_empty() {
        return Some(
            robominer_domain::rejection_messages::program_source_write_rejection_player_message(
                robominer_db::ProgramSourceWriteRejection::EmptySourceCode,
            ),
        );
    }
    None
}

fn format_program_source_apply_message(applied: &robominer_db::AppliedProgramSource) -> String {
    robominer_domain::rejection_messages::format_program_source_apply_player_message(applied)
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
