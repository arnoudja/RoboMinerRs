//! Shared fixtures for `edit_code_page` unit tests.

use std::collections::HashMap;

use crate::Request;
use crate::session::format_authenticated_cookie;

use super::super::{EditCodePageState, EditCodeProgramSource};

pub(super) fn authenticated_request(path: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: path.to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([(
            "cookie".to_string(),
            format_authenticated_cookie(42, "Player"),
        )]),
    }
}

pub(super) fn sample_edit_code_state(
    selected_program_source_id: i64,
    selected_program_source: EditCodeProgramSource,
    message: Option<String>,
) -> EditCodePageState {
    EditCodePageState {
        selected_program_source_id,
        selected_program_source,
        program_sources: vec![robominer_db::ProgramSourceStateRecord {
            source: robominer_db::ProgramSourceRecord {
                id: 11,
                user_id: 1,
                source_name: "Source <One>".to_string(),
                source_code: Some("move(1);\n// <mine>\nmine();".to_string()),
                verified: false,
                compiled_size: 12,
                error_description: "Compile <error>".to_string(),
            },
            linked_robot_count: 0,
        }],
        message,
        pending_claim_count: 0,

        claimed_results: robominer_db::ClaimedUserResults {
            claimed_queues: 0,
            ore_rewards: vec![],
        },
        prefer_stored_selection: true,
    }
}
