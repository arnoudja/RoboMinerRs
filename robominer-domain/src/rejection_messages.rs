//! User-facing strings for database mutation rejections.
//!
//! Web pages use the `*_player_message` helpers; engine CLI commands use the `*_cli_message`
//! helpers. Shared copy lives in a single function when both surfaces use the same text.
//!
//! Typed rejection enums live in `robominer-db`; this module owns the prose. See
//! `CONTRIBUTING.md` (“Crate boundary: robominer-db vs robominer-domain”).

pub fn create_user_rejection_player_message(
    rejection: robominer_db::CreateUserRejection,
) -> &'static str {
    match rejection {
        robominer_db::CreateUserRejection::InvalidUsername => "Invalid username",
        robominer_db::CreateUserRejection::InvalidEmail => "Invalid e-mail address",
        robominer_db::CreateUserRejection::InvalidPassword => {
            "The password doesn't meet the requirements"
        }
        robominer_db::CreateUserRejection::DuplicateUsername => {
            "Username already taken, please choose another one"
        }
        robominer_db::CreateUserRejection::DuplicateEmail => {
            "You already have an account, please login using your e-mail address"
        }
        robominer_db::CreateUserRejection::InitialAchievementRejected(_) => {
            "Unable to initialise new user achievements"
        }
    }
}

pub fn create_user_rejection_cli_message(rejection: robominer_db::CreateUserRejection) -> String {
    match rejection {
        robominer_db::CreateUserRejection::InvalidUsername => "invalid username".to_string(),
        robominer_db::CreateUserRejection::InvalidEmail => "invalid email".to_string(),
        robominer_db::CreateUserRejection::InvalidPassword => "invalid password".to_string(),
        robominer_db::CreateUserRejection::DuplicateUsername => "duplicate username".to_string(),
        robominer_db::CreateUserRejection::DuplicateEmail => "duplicate email".to_string(),
        robominer_db::CreateUserRejection::InitialAchievementRejected(rejection) => format!(
            "initial achievement rejected: {}",
            claim_achievement_step_rejection_message(rejection)
        ),
    }
}

pub fn update_user_account_rejection_player_message(
    rejection: robominer_db::UpdateUserAccountRejection,
) -> &'static str {
    match rejection {
        robominer_db::UpdateUserAccountRejection::UnknownUser => "Unknown user",
        robominer_db::UpdateUserAccountRejection::InvalidUsername => "Invalid username",
        robominer_db::UpdateUserAccountRejection::InvalidEmail => "Invalid e-mail address",
        robominer_db::UpdateUserAccountRejection::InvalidPassword => "Invalid password",
        robominer_db::UpdateUserAccountRejection::DuplicateUsername => {
            "That username is already taken"
        }
        robominer_db::UpdateUserAccountRejection::DuplicateEmail => {
            "Only one account per e-mail address is allowed"
        }
    }
}

pub fn update_user_account_rejection_cli_message(
    rejection: robominer_db::UpdateUserAccountRejection,
) -> &'static str {
    match rejection {
        robominer_db::UpdateUserAccountRejection::UnknownUser => "unknown user",
        robominer_db::UpdateUserAccountRejection::InvalidUsername => "invalid username",
        robominer_db::UpdateUserAccountRejection::InvalidEmail => "invalid email",
        robominer_db::UpdateUserAccountRejection::InvalidPassword => "invalid password",
        robominer_db::UpdateUserAccountRejection::DuplicateUsername => "duplicate username",
        robominer_db::UpdateUserAccountRejection::DuplicateEmail => "duplicate email",
    }
}

pub fn verify_login_rejection_cli_message(
    rejection: robominer_db::VerifyLoginRejection,
) -> &'static str {
    match rejection {
        robominer_db::VerifyLoginRejection::UnknownUser => "unknown user",
        robominer_db::VerifyLoginRejection::InvalidPassword => "invalid password",
    }
}

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

pub fn enqueue_mining_rejection_player_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    match rejection {
        robominer_db::EnqueueMiningRejection::UnknownRobot => "Unknown robot",
        robominer_db::EnqueueMiningRejection::UnknownMiningArea => "Unknown mining area",
        robominer_db::EnqueueMiningRejection::MiningAreaUnavailable => {
            "Unable to add to the mining queue: The mining area is not available."
        }
        robominer_db::EnqueueMiningRejection::QueueFull => {
            "Unable to add to the mining queue: The mining queue is full."
        }
        robominer_db::EnqueueMiningRejection::InsufficientFunds => {
            "Unable to add to the mining queue: You do not have enough funds to pay the mining costs."
        }
    }
}

pub fn enqueue_mining_rejection_cli_message(
    rejection: robominer_db::EnqueueMiningRejection,
) -> &'static str {
    match rejection {
        robominer_db::EnqueueMiningRejection::UnknownRobot => "unknown robot",
        robominer_db::EnqueueMiningRejection::UnknownMiningArea => "unknown mining area",
        robominer_db::EnqueueMiningRejection::MiningAreaUnavailable => {
            "mining area is not available to user"
        }
        robominer_db::EnqueueMiningRejection::QueueFull => "mining queue is full",
        robominer_db::EnqueueMiningRejection::InsufficientFunds => {
            "insufficient funds to pay mining costs"
        }
    }
}

pub fn cancel_mining_queue_rejection_player_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    match rejection {
        robominer_db::CancelMiningQueueRejection::UnknownQueue => "Unknown mining queue item.",
        robominer_db::CancelMiningQueueRejection::WrongOwner => {
            "Unable to cancel mining queue item."
        }
        robominer_db::CancelMiningQueueRejection::NotCancelable => {
            "Unable to cancel mining queue item: The mining queue item is not cancelable."
        }
        robominer_db::CancelMiningQueueRejection::RefundWouldClamp => {
            "Unable to cancel mining queue item: refund would exceed your wallet maximum."
        }
    }
}

pub fn cancel_mining_queue_rejection_cli_message(
    rejection: robominer_db::CancelMiningQueueRejection,
) -> &'static str {
    match rejection {
        robominer_db::CancelMiningQueueRejection::UnknownQueue => "unknown mining queue item",
        robominer_db::CancelMiningQueueRejection::WrongOwner => {
            "mining queue item belongs to another user"
        }
        robominer_db::CancelMiningQueueRejection::NotCancelable => {
            "mining queue item is not cancelable"
        }
        robominer_db::CancelMiningQueueRejection::RefundWouldClamp => {
            "cancel would clamp ore refund past maxAllowed"
        }
    }
}

pub fn claim_achievement_step_rejection_message(
    rejection: robominer_db::ClaimAchievementStepRejection,
) -> &'static str {
    match rejection {
        robominer_db::ClaimAchievementStepRejection::UnknownUserAchievement => {
            "unknown user achievement"
        }
        robominer_db::ClaimAchievementStepRejection::NoNextStep => "no next achievement step",
        robominer_db::ClaimAchievementStepRejection::RequirementsNotMet => {
            "achievement requirements are not met"
        }
        robominer_db::ClaimAchievementStepRejection::MissingDefaultRobotPart => {
            "missing default robot part"
        }
        robominer_db::ClaimAchievementStepRejection::InvalidDefaultRobotConfiguration => {
            "invalid default robot configuration"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_and_cli_shop_messages_match() {
        assert_eq!(
            robot_part_transaction_rejection_message(
                robominer_db::RobotPartTransactionRejection::InsufficientFunds
            ),
            "insufficient funds to pay robot part costs"
        );
    }

    #[test]
    fn achievement_messages_are_shared() {
        assert_eq!(
            claim_achievement_step_rejection_message(
                robominer_db::ClaimAchievementStepRejection::RequirementsNotMet
            ),
            "achievement requirements are not met"
        );
    }

    #[test]
    fn create_user_player_message_hides_nested_achievement_detail() {
        assert_eq!(
            create_user_rejection_player_message(
                robominer_db::CreateUserRejection::InitialAchievementRejected(
                    robominer_db::ClaimAchievementStepRejection::RequirementsNotMet
                )
            ),
            "Unable to initialise new user achievements"
        );
    }

    #[test]
    fn create_user_cli_message_includes_nested_achievement_detail() {
        assert_eq!(
            create_user_rejection_cli_message(
                robominer_db::CreateUserRejection::InitialAchievementRejected(
                    robominer_db::ClaimAchievementStepRejection::RequirementsNotMet
                )
            ),
            "initial achievement rejected: achievement requirements are not met"
        );
    }

    #[test]
    fn every_rejection_message_arm_is_non_empty() {
        for rejection in [
            robominer_db::CreateUserRejection::InvalidUsername,
            robominer_db::CreateUserRejection::InvalidEmail,
            robominer_db::CreateUserRejection::InvalidPassword,
            robominer_db::CreateUserRejection::DuplicateUsername,
            robominer_db::CreateUserRejection::DuplicateEmail,
            robominer_db::CreateUserRejection::InitialAchievementRejected(
                robominer_db::ClaimAchievementStepRejection::NoNextStep,
            ),
        ] {
            assert!(!create_user_rejection_player_message(rejection).is_empty());
            assert!(!create_user_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::UpdateUserAccountRejection::UnknownUser,
            robominer_db::UpdateUserAccountRejection::InvalidUsername,
            robominer_db::UpdateUserAccountRejection::InvalidEmail,
            robominer_db::UpdateUserAccountRejection::InvalidPassword,
            robominer_db::UpdateUserAccountRejection::DuplicateUsername,
            robominer_db::UpdateUserAccountRejection::DuplicateEmail,
        ] {
            assert!(!update_user_account_rejection_player_message(rejection).is_empty());
            assert!(!update_user_account_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::VerifyLoginRejection::UnknownUser,
            robominer_db::VerifyLoginRejection::InvalidPassword,
        ] {
            assert!(!verify_login_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::ProgramSourceWriteRejection::UnknownUser,
            robominer_db::ProgramSourceWriteRejection::UnknownProgramSource,
            robominer_db::ProgramSourceWriteRejection::SourceInUse,
            robominer_db::ProgramSourceWriteRejection::EmptySourceName,
            robominer_db::ProgramSourceWriteRejection::EmptySourceCode,
        ] {
            assert!(!program_source_write_rejection_player_message(rejection).is_empty());
            assert!(!program_source_write_rejection_cli_message(rejection).is_empty());
        }

        for reason in [
            robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory,
            robominer_db::ProgramSourceApplyWarningReason::RobotBusy,
        ] {
            assert!(!program_source_apply_warning_message(reason).is_empty());
        }

        for rejection in [
            robominer_db::UpdateRobotConfigRejection::UnknownRobot,
            robominer_db::UpdateRobotConfigRejection::ChangeAlreadyPending,
            robominer_db::UpdateRobotConfigRejection::InvalidRobotName,
            robominer_db::UpdateRobotConfigRejection::UnknownProgramSource,
            robominer_db::UpdateRobotConfigRejection::UnknownRobotPart,
            robominer_db::UpdateRobotConfigRejection::ProgramTooLarge,
            robominer_db::UpdateRobotConfigRejection::NoUnassignedRobotPart,
            robominer_db::UpdateRobotConfigRejection::InvalidRobotPartConfiguration,
        ] {
            assert!(!update_robot_config_rejection_player_message(rejection).is_empty());
            assert!(!update_robot_config_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::RobotPartTransactionRejection::UnknownUser,
            robominer_db::RobotPartTransactionRejection::UnknownRobotPart,
            robominer_db::RobotPartTransactionRejection::InsufficientFunds,
            robominer_db::RobotPartTransactionRejection::NoUnassignedRobotPart,
        ] {
            assert!(!robot_part_transaction_rejection_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::EnqueueMiningRejection::UnknownRobot,
            robominer_db::EnqueueMiningRejection::UnknownMiningArea,
            robominer_db::EnqueueMiningRejection::MiningAreaUnavailable,
            robominer_db::EnqueueMiningRejection::QueueFull,
            robominer_db::EnqueueMiningRejection::InsufficientFunds,
        ] {
            assert!(!enqueue_mining_rejection_player_message(rejection).is_empty());
            assert!(!enqueue_mining_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::CancelMiningQueueRejection::UnknownQueue,
            robominer_db::CancelMiningQueueRejection::WrongOwner,
            robominer_db::CancelMiningQueueRejection::NotCancelable,
            robominer_db::CancelMiningQueueRejection::RefundWouldClamp,
        ] {
            assert!(!cancel_mining_queue_rejection_player_message(rejection).is_empty());
            assert!(!cancel_mining_queue_rejection_cli_message(rejection).is_empty());
        }

        for rejection in [
            robominer_db::ClaimAchievementStepRejection::UnknownUserAchievement,
            robominer_db::ClaimAchievementStepRejection::NoNextStep,
            robominer_db::ClaimAchievementStepRejection::RequirementsNotMet,
            robominer_db::ClaimAchievementStepRejection::MissingDefaultRobotPart,
            robominer_db::ClaimAchievementStepRejection::InvalidDefaultRobotConfiguration,
        ] {
            assert!(!claim_achievement_step_rejection_message(rejection).is_empty());
        }
    }

    #[test]
    fn format_program_source_apply_player_message_covers_outcomes() {
        assert_eq!(
            format_program_source_apply_player_message(&robominer_db::AppliedProgramSource {
                applied_robots: 0,
                warnings: Vec::new(),
            }),
            "Unable to update robots: program has a compile error."
        );

        assert_eq!(
            format_program_source_apply_player_message(&robominer_db::AppliedProgramSource {
                applied_robots: 2,
                warnings: Vec::new(),
            }),
            "Updated 2 robot(s)."
        );

        assert_eq!(
            format_program_source_apply_player_message(&robominer_db::AppliedProgramSource {
                applied_robots: 0,
                warnings: vec![robominer_db::ProgramSourceApplyWarning {
                    robot_name: "BusyBot".to_string(),
                    reason: robominer_db::ProgramSourceApplyWarningReason::RobotBusy,
                }],
            }),
            "Unable to update linked robots. Unable to update BusyBot: The robot is busy."
        );

        assert_eq!(
            format_program_source_apply_player_message(&robominer_db::AppliedProgramSource {
                applied_robots: 1,
                warnings: vec![robominer_db::ProgramSourceApplyWarning {
                    robot_name: "Tiny".to_string(),
                    reason: robominer_db::ProgramSourceApplyWarningReason::NotEnoughMemory,
                }],
            }),
            "Updated 1 robot(s). Unable to update Tiny: Not enough memory."
        );
    }
}
