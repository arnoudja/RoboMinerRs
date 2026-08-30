//! User-facing strings for database mutation rejections.
//!
//! Web pages use the `*_player_message` helpers; engine CLI commands use the `*_cli_message`
//! helpers. Shared copy lives in a single function when both surfaces use the same text.
//!
//! Typed rejection enums live in `robominer-db`; this module owns the prose. See
//! `CONTRIBUTING.md` (“Crate boundary: robominer-db vs robominer-domain”).

mod achievements;
mod mining;
mod program;
mod user;

pub use achievements::*;
pub use mining::*;
pub use program::*;
pub use user::*;
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
            robominer_db::ProgramSourceWriteRejection::SourceCodeTooLong,
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
