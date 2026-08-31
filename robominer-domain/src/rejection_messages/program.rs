use super::Audience;

pub fn program_source_write_rejection_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
    audience: Audience,
) -> &'static str {
    match (rejection, audience) {
        (robominer_db::ProgramSourceWriteRejection::UnknownUser, Audience::Player) => {
            "Unknown user."
        }
        (robominer_db::ProgramSourceWriteRejection::UnknownUser, Audience::Cli) => "unknown user",
        (robominer_db::ProgramSourceWriteRejection::UnknownProgramSource, Audience::Player) => {
            "Unknown program source."
        }
        (robominer_db::ProgramSourceWriteRejection::UnknownProgramSource, Audience::Cli) => {
            "unknown program source"
        }
        (robominer_db::ProgramSourceWriteRejection::SourceInUse, Audience::Player) => {
            "Unable to delete program source because it is used by a robot."
        }
        (robominer_db::ProgramSourceWriteRejection::SourceInUse, Audience::Cli) => {
            "program source is still linked to a robot"
        }
        (robominer_db::ProgramSourceWriteRejection::EmptySourceName, Audience::Player) => {
            "Program name may not be empty."
        }
        (robominer_db::ProgramSourceWriteRejection::EmptySourceName, Audience::Cli) => {
            "empty source name"
        }
        (robominer_db::ProgramSourceWriteRejection::EmptySourceCode, Audience::Player) => {
            "Program source may not be empty."
        }
        (robominer_db::ProgramSourceWriteRejection::EmptySourceCode, Audience::Cli) => {
            "empty source code"
        }
        (robominer_db::ProgramSourceWriteRejection::SourceCodeTooLong, Audience::Player) => {
            "Program source is too long."
        }
        (robominer_db::ProgramSourceWriteRejection::SourceCodeTooLong, Audience::Cli) => {
            "source code too long"
        }
    }
}

pub fn program_source_write_rejection_player_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
) -> &'static str {
    program_source_write_rejection_message(rejection, Audience::Player)
}

pub fn program_source_write_rejection_cli_message(
    rejection: robominer_db::ProgramSourceWriteRejection,
) -> &'static str {
    program_source_write_rejection_message(rejection, Audience::Cli)
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

pub fn update_robot_config_rejection_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
    audience: Audience,
) -> &'static str {
    match (rejection, audience) {
        (robominer_db::UpdateRobotConfigRejection::UnknownRobot, Audience::Player) => {
            "Unknown robot"
        }
        (robominer_db::UpdateRobotConfigRejection::UnknownRobot, Audience::Cli) => "unknown robot",
        (robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending, Audience::Player) => {
            "Changes are already pending for this robot."
        }
        (robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending, Audience::Cli) => {
            "robot already has pending changes"
        }
        (robominer_db::UpdateRobotConfigRejection::InvalidRobotName, Audience::Player) => {
            "Invalid robot name."
        }
        (robominer_db::UpdateRobotConfigRejection::InvalidRobotName, Audience::Cli) => {
            "invalid robot name"
        }
        (robominer_db::UpdateRobotConfigRejection::UnknownProgramSource, Audience::Player) => {
            "Unknown program source."
        }
        (robominer_db::UpdateRobotConfigRejection::UnknownProgramSource, Audience::Cli) => {
            "unknown program source"
        }
        (robominer_db::UpdateRobotConfigRejection::UnknownRobotPart, Audience::Player) => {
            "Unknown robot part."
        }
        (robominer_db::UpdateRobotConfigRejection::UnknownRobotPart, Audience::Cli) => {
            "unknown robot part"
        }
        (robominer_db::UpdateRobotConfigRejection::ProgramTooLarge, Audience::Player) => {
            "Not enough memory available."
        }
        (robominer_db::UpdateRobotConfigRejection::ProgramTooLarge, Audience::Cli) => {
            "program source does not fit in memory"
        }
        (robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart, Audience::Player) => {
            "No unassigned robot part is available."
        }
        (robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart, Audience::Cli) => {
            "no unassigned robot part is available"
        }
        (
            robominer_db::UpdateRobotConfigRejection::InvalidRobotPartConfiguration,
            Audience::Player,
        ) => "Invalid robot part configuration.",
        (
            robominer_db::UpdateRobotConfigRejection::InvalidRobotPartConfiguration,
            Audience::Cli,
        ) => "invalid robot part configuration",
    }
}

pub fn update_robot_config_rejection_player_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
) -> &'static str {
    update_robot_config_rejection_message(rejection, Audience::Player)
}

pub fn update_robot_config_rejection_cli_message(
    rejection: robominer_db::UpdateRobotConfigRejection,
) -> &'static str {
    update_robot_config_rejection_message(rejection, Audience::Cli)
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
