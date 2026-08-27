use super::super::selected_edit_code_source;

#[test]
fn edit_code_new_program_selection_does_not_fall_back_to_first_source() {
    let sources = vec![robominer_db::ProgramSourceStateRecord {
        source: robominer_db::ProgramSourceRecord {
            id: 11,
            user_id: 1,
            source_name: "Existing".to_string(),
            source_code: Some("move(1);".to_string()),
            verified: true,
            compiled_size: 4,
            error_description: String::new(),
        },
        linked_robot_count: 0,
    }];

    assert_eq!(
        selected_edit_code_source(&sources, None).map(|state| state.source.id),
        Some(11)
    );
    assert_eq!(
        selected_edit_code_source(&sources, Some(11)).map(|state| state.source.id),
        Some(11)
    );
    assert!(
        selected_edit_code_source(&sources, Some(-1)).is_none(),
        "New program must render the default program, not the first existing source"
    );
}

#[test]
fn edit_code_create_selection_prefers_created_source_id() {
    let sources = vec![
        robominer_db::ProgramSourceStateRecord {
            source: robominer_db::ProgramSourceRecord {
                id: 11,
                user_id: 1,
                source_name: "Existing".to_string(),
                source_code: Some("move(1);".to_string()),
                verified: true,
                compiled_size: 4,
                error_description: String::new(),
            },
            linked_robot_count: 0,
        },
        robominer_db::ProgramSourceStateRecord {
            source: robominer_db::ProgramSourceRecord {
                id: 42,
                user_id: 1,
                source_name: "Just created".to_string(),
                source_code: Some("mine();".to_string()),
                verified: true,
                compiled_size: 2,
                error_description: String::new(),
            },
            linked_robot_count: 0,
        },
    ];

    assert_eq!(
        selected_edit_code_source(&sources, Some(42)).map(|state| state.source.id),
        Some(42),
        "after create, selection must follow the new program id"
    );
}

#[test]
fn edit_code_rejection_messages_are_user_facing() {
    assert_eq!(
        robominer_domain::rejection_messages::program_source_write_rejection_player_message(
            robominer_db::ProgramSourceWriteRejection::SourceInUse
        ),
        "Unable to delete program source because it is used by a robot."
    );
    assert_eq!(
        robominer_domain::rejection_messages::program_source_write_rejection_player_message(
            robominer_db::ProgramSourceWriteRejection::EmptySourceName
        ),
        "Program name may not be empty."
    );
    assert_eq!(
        robominer_domain::rejection_messages::program_source_write_rejection_player_message(
            robominer_db::ProgramSourceWriteRejection::EmptySourceCode
        ),
        "Program source may not be empty."
    );
}
