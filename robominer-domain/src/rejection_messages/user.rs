use super::Audience;
use super::achievements::claim_achievement_step_rejection_message;

pub fn create_user_rejection_player_message(
    rejection: robominer_db::CreateUserRejection,
) -> &'static str {
    match create_user_rejection_message(rejection, Audience::Player) {
        std::borrow::Cow::Borrowed(message) => message,
        std::borrow::Cow::Owned(_) => {
            unreachable!("player create-user messages are always static")
        }
    }
}

pub fn create_user_rejection_cli_message(rejection: robominer_db::CreateUserRejection) -> String {
    create_user_rejection_message(rejection, Audience::Cli).into_owned()
}

fn create_user_rejection_message(
    rejection: robominer_db::CreateUserRejection,
    audience: Audience,
) -> std::borrow::Cow<'static, str> {
    match (rejection, audience) {
        (robominer_db::CreateUserRejection::InvalidUsername, Audience::Player) => {
            "Invalid username".into()
        }
        (robominer_db::CreateUserRejection::InvalidUsername, Audience::Cli) => {
            "invalid username".into()
        }
        (robominer_db::CreateUserRejection::InvalidEmail, Audience::Player) => {
            "Invalid e-mail address".into()
        }
        (robominer_db::CreateUserRejection::InvalidEmail, Audience::Cli) => "invalid email".into(),
        (robominer_db::CreateUserRejection::InvalidPassword, Audience::Player) => {
            "The password doesn't meet the requirements".into()
        }
        (robominer_db::CreateUserRejection::InvalidPassword, Audience::Cli) => {
            "invalid password".into()
        }
        // Generic duplicate copy for public signup (avoids account enumeration).
        (robominer_db::CreateUserRejection::DuplicateUsername, Audience::Player) => {
            "Could not create that account. Try a different username or e-mail, or log in if you already have one.".into()
        }
        (robominer_db::CreateUserRejection::DuplicateUsername, Audience::Cli) => {
            "duplicate username".into()
        }
        (robominer_db::CreateUserRejection::DuplicateEmail, Audience::Player) => {
            "Could not create that account. Try a different username or e-mail, or log in if you already have one.".into()
        }
        (robominer_db::CreateUserRejection::DuplicateEmail, Audience::Cli) => {
            "duplicate email".into()
        }
        (robominer_db::CreateUserRejection::InitialAchievementRejected(_), Audience::Player) => {
            "Unable to initialise new user achievements".into()
        }
        (
            robominer_db::CreateUserRejection::InitialAchievementRejected(rejection),
            Audience::Cli,
        ) => format!(
            "initial achievement rejected: {}",
            claim_achievement_step_rejection_message(rejection)
        )
        .into(),
    }
}

pub fn update_user_account_rejection_message(
    rejection: robominer_db::UpdateUserAccountRejection,
    audience: Audience,
) -> &'static str {
    match (rejection, audience) {
        (robominer_db::UpdateUserAccountRejection::UnknownUser, Audience::Player) => "Unknown user",
        (robominer_db::UpdateUserAccountRejection::UnknownUser, Audience::Cli) => "unknown user",
        (robominer_db::UpdateUserAccountRejection::InvalidUsername, Audience::Player) => {
            "Invalid username"
        }
        (robominer_db::UpdateUserAccountRejection::InvalidUsername, Audience::Cli) => {
            "invalid username"
        }
        (robominer_db::UpdateUserAccountRejection::InvalidEmail, Audience::Player) => {
            "Invalid e-mail address"
        }
        (robominer_db::UpdateUserAccountRejection::InvalidEmail, Audience::Cli) => "invalid email",
        (robominer_db::UpdateUserAccountRejection::InvalidPassword, Audience::Player) => {
            "Invalid password"
        }
        (robominer_db::UpdateUserAccountRejection::InvalidPassword, Audience::Cli) => {
            "invalid password"
        }
        // Generic duplicate copy (avoids account enumeration).
        (robominer_db::UpdateUserAccountRejection::DuplicateUsername, Audience::Player) => {
            "Could not update that account. Try a different username or e-mail."
        }
        (robominer_db::UpdateUserAccountRejection::DuplicateUsername, Audience::Cli) => {
            "duplicate username"
        }
        (robominer_db::UpdateUserAccountRejection::DuplicateEmail, Audience::Player) => {
            "Could not update that account. Try a different username or e-mail."
        }
        (robominer_db::UpdateUserAccountRejection::DuplicateEmail, Audience::Cli) => {
            "duplicate email"
        }
    }
}

pub fn update_user_account_rejection_player_message(
    rejection: robominer_db::UpdateUserAccountRejection,
) -> &'static str {
    update_user_account_rejection_message(rejection, Audience::Player)
}

pub fn update_user_account_rejection_cli_message(
    rejection: robominer_db::UpdateUserAccountRejection,
) -> &'static str {
    update_user_account_rejection_message(rejection, Audience::Cli)
}

pub fn verify_login_rejection_cli_message(
    rejection: robominer_db::VerifyLoginRejection,
) -> &'static str {
    match rejection {
        robominer_db::VerifyLoginRejection::UnknownUser => "unknown user",
        robominer_db::VerifyLoginRejection::InvalidPassword => "invalid password",
    }
}
