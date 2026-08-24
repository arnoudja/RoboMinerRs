use super::achievements::claim_achievement_step_rejection_message;

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
