pub fn program_source_write_rejection_player_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
) -> &'static str {
    match rejection {
        robominer_db::ProgramSourceWriteRejection::UnknownUser => "Unknown user.",
        robominer_db::ProgramSourceWriteRejection::UnknownProgramSource => {
            "Unknown program source."
        }
        robominer_db::ProgramSourceWriteRejection::SourceInUse => {
            "Unable to delete program source because it is used by a robot."
        }
        robominer_db::ProgramSourceWriteRejection::EmptySourceName => {
            "Program name may not be empty."
        }
        robominer_db::ProgramSourceWriteRejection::EmptySourceCode => {
            "Program source may not be empty."
        }
    }
}

pub fn program_source_write_rejection_cli_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
) -> &'static str {
    match rejection {
        robominer_db::ProgramSourceWriteRejection::UnknownUser => "unknown user",
        robominer_db::ProgramSourceWriteRejection::UnknownProgramSource => "unknown program source",
        robominer_db::ProgramSourceWriteRejection::SourceInUse => {
            "program source is still linked to a robot"
        }
        robominer_db::ProgramSourceWriteRejection::EmptySourceName => "empty source name",
        robominer_db::ProgramSourceWriteRejection::EmptySourceCode => "empty source code",
    }
}

pub fn program_source_apply_warning_message(
    reason: robominer_db::ProgramSourceApplyWarningReason,
) -> &'static str {
    match reason {
        robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory => "Not enough memory.",
        robominer_db::ProgramSourceApplyWarningReason::RobotBusy => "The robot is busy.",
    }
}

pub fn format_program_source_apply_player_message(
    applied: &robominer_db::AppliedProgramSource,
) -> String {
    if applied.applied_robots == 0 && applied.warnings.is_empty() {
        return "Unable to update robots: program has a compile error.".to_string();
    }

    let mut parts = Vec::new();
    if applied.applied_robots > 0 {
        parts.push(format!("Updated {} robot(s).", applied.applied_robots));
    } else {
        parts.push("Unable to update linked robots.".to_string());
    }

    for warning in &applied.warnings {
        parts.push(format!(
            "Unable to update {}: {}",
            warning.robot_name,
            program_source_apply_warning_message(warning.reason)
        ));
    }

    parts.join(" ")
}

pub fn update_robot_config_rejection_player_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
) -> &'static str {
    match rejection {
        robominer_db::UpdateRobotConfigRejection::UnknownRobot => "Unknown robot",
        robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending => {
            "Changes are already pending for this robot."
        }
        robominer_db::UpdateRobotConfigRejection::InvalidRobotName => "Invalid robot name.",
        robominer_db::UpdateRobotConfigRejection::UnknownProgramSource => "Unknown program source.",
        robominer_db::UpdateRobotConfigRejection::UnknownRobotPart => "Unknown robot part.",
        robominer_db::UpdateRobotConfigRejection::ProgramTooLarge => "Not enough memory available.",
        robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart => {
            "No unassigned robot part is available."
        }
        robominer_db::UpdateRobotConfigRejection::InvalidRobotPartConfiguration => {
            "Invalid robot part configuration."
        }
    }
}

pub fn update_robot_config_rejection_cli_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
) -> &'static str {
    match rejection {
        robominer_db::UpdateRobotConfigRejection::UnknownRobot => "unknown robot",
        robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending => {
            "robot already has pending changes"
        }
        robominer_db::UpdateRobotConfigRejection::InvalidRobotName => "invalid robot name",
        robominer_db::UpdateRobotConfigRejection::UnknownProgramSource => "unknown program source",
        robominer_db::UpdateRobotConfigRejection::UnknownRobotPart => "unknown robot part",
        robominer_db::UpdateRobotConfigRejection::ProgramTooLarge => {
            "program source does not fit in memory"
        }
        robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart => {
            "no unassigned robot part is available"
        }
        robominer_db::UpdateRobotConfigRejection::InvalidRobotPartConfiguration => {
            "invalid robot part configuration"
        }
    }
}

pub fn robot_part_transaction_rejection_message(
    rejection: robominer_db::RobotPartTransactionRejection,
) -> &'static str {
    match rejection {
        robominer_db::RobotPartTransactionRejection::UnknownUser => "unknown user",
        robominer_db::RobotPartTransactionRejection::UnknownRobotPart => "unknown robot part",
        robominer_db::RobotPartTransactionRejection::InsufficientFunds => {
            "insufficient funds to pay robot part costs"
        }
        robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart => {
            "no unassigned robot part is available"
        }
    }
}
