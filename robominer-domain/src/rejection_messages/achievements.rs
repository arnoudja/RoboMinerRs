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
