use crate::rate_limit::{
    auth_attempt_is_rate_limited, client_ip, log_auth_failure, record_auth_attempt,
};
use crate::session;
use crate::{Request, Response, ServerConfig, is_post, login_redirect, session_username};

#[derive(Debug)]
pub(super) struct AccountPageState {
    pub(super) username: String,
    pub(super) email: String,
    pub(super) current_username: String,
    pub(super) message: Option<String>,
    pub(super) error_message: Option<String>,
    pub(super) reissue_session_version: Option<i32>,
}

pub(super) async fn account_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(user_id) = crate::request_user_id(request) else {
        return login_redirect(request);
    };
    if let Some(response) = crate::csrf::reject_invalid_csrf(request, user_id) {
        return response;
    }

    // Account updates always verify the current password (Argon2). Rate-limit before DB work.
    if is_account_update_post(request) {
        let ip = client_ip(request, config.trust_proxy);
        let account_key = account_rate_limit_key(user_id);
        if auth_attempt_is_rate_limited(&ip, &account_key) {
            log_auth_failure(&ip, &account_key, "rate_limited");
            return Response::too_many_requests(
                "Too many account password checks. Please try again later.",
            );
        }
        record_auth_attempt(&ip, &account_key);
    }

    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Account requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let result = load_account_page_state(pool, user_id, request).await;

    match result {
        Ok(state) => {
            let reissue_session_version = state.reissue_session_version;
            let username_for_cookie = state.current_username.clone();
            let mut response = crate::csrf::html_with_csrf(
                request,
                user_id,
                render::render_account_page(
                    crate::app_shell::hud_markup(request, config)
                        .await
                        .as_deref(),
                    &state,
                ),
            );
            if let Some(session_version) = reissue_session_version {
                response = reissue_session_cookies(response, user_id, session_version, &username_for_cookie);
            }
            response
        }
        Err(error) => Response::service_unavailable(format!("Unable to load account: {error}")),
    }
}

fn reissue_session_cookies(
    mut response: Response,
    user_id: i64,
    session_version: i32,
    username: &str,
) -> Response {
    response
        .headers
        .retain(|(name, value)| !(*name == "Set-Cookie" && value.starts_with("robominer_session=")));
    response
        .with_header(
            "Set-Cookie",
            session::session_set_cookie_header(user_id, false, session_version),
        )
        .with_header(
            "Set-Cookie",
            format!(
                "robominer_username={}; Path=/; SameSite=Lax{}",
                session::cookie_encode(username),
                session::secure_cookie_suffix()
            ),
        )
}

fn is_account_update_post(request: &Request) -> bool {
    is_post(request) && request.form.contains_key("username")
}

fn account_rate_limit_key(user_id: i64) -> String {
    format!("user:{user_id}")
}

async fn load_account_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<AccountPageState, robominer_domain::DomainError> {
    robominer_db::claim_user_results(pool, user_id).await?;

    let Some(current_user) = robominer_db::get_user_by_id(pool, user_id).await? else {
        return Ok(AccountPageState {
            username: String::new(),
            email: String::new(),
            current_username: session_username(request),
            message: None,
            error_message: Some("Unknown user".to_string()),
            reissue_session_version: None,
        });
    };

    let mut username = current_user.username.clone();
    let mut email = current_user.email.clone();
    let mut current_username = current_user.username.clone();
    let mut message = None;
    let mut error_message = None;
    let mut reissue_session_version = None;

    if is_post(request) && request.form.contains_key("username") {
        let submitted_username = request.form.get("username").cloned().unwrap_or_default();
        let submitted_email = request.form.get("email").cloned().unwrap_or_default();
        let current_password = request
            .form
            .get("currentpassword")
            .cloned()
            .unwrap_or_default();
        let new_password = request.form.get("newpassword").cloned().unwrap_or_default();
        let confirm_password = request
            .form
            .get("confirmpassword")
            .cloned()
            .unwrap_or_default();

        let password_verified = robominer_db::verify_user_password(
            pool,
            robominer_db::VerifyUserPasswordRequest {
                user_id,
                password: current_password,
            },
        )
        .await?
        .is_ok();

        if !password_verified {
            username = submitted_username;
            email = submitted_email;
            error_message = Some("Your current password doesn't match".to_string());
        } else if new_password != confirm_password {
            username = submitted_username;
            email = submitted_email;
            error_message = Some(account_password_mismatch_message().to_string());
        } else {
            let password = if !new_password.is_empty() {
                Some(new_password)
            } else {
                None
            };
            let update_result = robominer_db::update_user_account(
                pool,
                robominer_db::UpdateUserAccountRequest {
                    user_id,
                    username: submitted_username.clone(),
                    email: submitted_email.clone(),
                    password,
                },
            )
            .await?;

            match update_result {
                Ok(updated) => {
                    message = Some("Account information updated".to_string());
                    if updated.password_changed {
                        reissue_session_version = Some(updated.session_version);
                    }
                    if let Some(updated_user) = robominer_db::get_user_by_id(pool, user_id).await? {
                        username = updated_user.username;
                        email = updated_user.email;
                        current_username = username.clone();
                    }
                }
                Err(rejection) => {
                    username = submitted_username;
                    email = submitted_email;
                    error_message =
                        Some(update_user_account_rejection_message(rejection).to_string());
                }
            }
        }
    }

    Ok(AccountPageState {
        username,
        email,
        current_username,
        message,
        error_message,
        reissue_session_version,
    })
}

pub(super) fn account_password_mismatch_message() -> &'static str {
    "The passwords do not match."
}

pub(super) fn update_user_account_rejection_message(
    rejection: robominer_db::UpdateUserAccountRejection,
) -> &'static str {
    robominer_domain::update_user_account_rejection_player_message(rejection)
}

mod render;

#[cfg(test)]
mod tests;
