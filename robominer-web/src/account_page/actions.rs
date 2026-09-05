//! Account update and logout-all-devices mutations.

use crate::{Request, is_post};

pub(super) struct AccountMutationResult {
    pub(super) message: Option<String>,
    pub(super) error_message: Option<String>,
    pub(super) reissue_session_version: Option<i32>,
    pub(super) submitted_username: Option<String>,
    pub(super) submitted_email: Option<String>,
}

pub(super) fn is_account_update_post(request: &Request) -> bool {
    is_post(request) && request.form.contains_key("username")
}

pub(super) fn is_logout_all_devices_post(request: &Request) -> bool {
    is_post(request) && request.form.contains_key("logoutAllDevices")
}

pub(super) async fn apply_account_mutations(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
    password_verified: bool,
) -> Result<Option<AccountMutationResult>, crate::page_context::PageLoadError> {
    if is_logout_all_devices_post(request) {
        if !password_verified {
            return Ok(Some(AccountMutationResult {
                message: None,
                error_message: Some("Your current password doesn't match".to_string()),
                reissue_session_version: None,
                submitted_username: None,
                submitted_email: None,
            }));
        }
        return Ok(Some(
            match robominer_db::users::bump_user_session_version(pool, user_id).await? {
                Some(session_version) => AccountMutationResult {
                    message: Some("Signed out of all other devices".to_string()),
                    error_message: None,
                    reissue_session_version: Some(session_version),
                    submitted_username: None,
                    submitted_email: None,
                },
                None => AccountMutationResult {
                    message: None,
                    error_message: Some("Unknown user".to_string()),
                    reissue_session_version: None,
                    submitted_username: None,
                    submitted_email: None,
                },
            },
        ));
    }

    if !is_account_update_post(request) {
        return Ok(None);
    }

    let submitted_username = request.form.get("username").cloned().unwrap_or_default();
    let submitted_email = request.form.get("email").cloned().unwrap_or_default();
    let new_password = request.form.get("newpassword").cloned().unwrap_or_default();
    let confirm_password = request
        .form
        .get("confirmpassword")
        .cloned()
        .unwrap_or_default();

    if !password_verified {
        return Ok(Some(AccountMutationResult {
            message: None,
            error_message: Some("Your current password doesn't match".to_string()),
            reissue_session_version: None,
            submitted_username: Some(submitted_username),
            submitted_email: Some(submitted_email),
        }));
    }

    if new_password != confirm_password {
        return Ok(Some(AccountMutationResult {
            message: None,
            error_message: Some(super::account_password_mismatch_message().to_string()),
            reissue_session_version: None,
            submitted_username: Some(submitted_username),
            submitted_email: Some(submitted_email),
        }));
    }

    let password = if !new_password.is_empty() {
        Some(new_password)
    } else {
        None
    };

    match robominer_db::update_user_account(
        pool,
        robominer_db::UpdateUserAccountRequest {
            user_id,
            username: submitted_username.clone(),
            email: submitted_email.clone(),
            password,
        },
    )
    .await?
    {
        robominer_db::DbOutcome::Success(updated) => Ok(Some(AccountMutationResult {
            message: Some("Account information updated".to_string()),
            error_message: None,
            reissue_session_version: updated.password_changed.then_some(updated.session_version),
            submitted_username: None,
            submitted_email: None,
        })),
        robominer_db::DbOutcome::Rejected(rejection) => Ok(Some(AccountMutationResult {
            message: None,
            error_message: Some(
                robominer_domain::rejection_messages::update_user_account_rejection_player_message(
                    rejection,
                )
                .to_string(),
            ),
            reissue_session_version: None,
            submitted_username: Some(submitted_username),
            submitted_email: Some(submitted_email),
        })),
    }
}
