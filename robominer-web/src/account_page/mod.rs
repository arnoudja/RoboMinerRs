use crate::rate_limit::{
    auth_attempt_is_rate_limited, client_ip, log_auth_failure, record_auth_attempt,
};
use crate::session;
use crate::{Request, Response, ServerConfig, is_post, session_username};

mod actions;
mod render;

#[cfg(test)]
mod tests;

use actions::{apply_account_mutations, is_account_update_post, is_logout_all_devices_post};

#[derive(Debug)]
pub(super) struct AccountPageState {
    pub(super) username: String,
    pub(super) email: String,
    pub(super) current_username: String,
    pub(super) message: Option<String>,
    pub(super) error_message: Option<String>,
    pub(super) reissue_session_version: Option<i32>,
}

pub(super) async fn account_page(
    request: &Request,
    config: &ServerConfig,
    session: crate::page_context::PageSession<'_>,
) -> Response {
    if is_account_update_post(request) || is_logout_all_devices_post(request) {
        let ip = client_ip(request, config.trust_proxy);
        let account_key = account_rate_limit_key(session.user_id);
        if auth_attempt_is_rate_limited(&ip, &account_key) {
            log_auth_failure(&ip, &account_key, "rate_limited");
            return Response::too_many_requests(
                "Too many account password checks. Please try again later.",
            );
        }
        record_auth_attempt(&ip, &account_key);
    }

    let result = load_account_page_state(session.pool, session.user_id, request).await;

    match result {
        Ok(state) => {
            let reissue_session_version = state.reissue_session_version;
            let username_for_cookie = state.current_username.clone();
            let user_id = session.user_id;
            let mut response = session
                .html_with_hud(request, config, |_username, hud| {
                    render::render_account_page(hud, &state)
                })
                .await;
            if let Some(session_version) = reissue_session_version {
                response = reissue_session_cookies(
                    response,
                    user_id,
                    session_version,
                    &username_for_cookie,
                );
            }
            response
        }
        Err(error) => crate::page_context::page_load_error("account", error),
    }
}

fn reissue_session_cookies(
    mut response: Response,
    user_id: i64,
    session_version: i32,
    username: &str,
) -> Response {
    let session_prefix = format!("{}=", session::session_cookie_name());
    response
        .headers
        .retain(|(name, value)| !(*name == "Set-Cookie" && value.starts_with(&session_prefix)));
    let response = response
        .with_header(
            "Set-Cookie",
            session::session_set_cookie_header(user_id, false, session_version),
        )
        .with_header("Set-Cookie", session::username_set_cookie_header(username));
    session::with_set_cookies(response, session::legacy_auth_cookie_clear_headers())
}

fn account_rate_limit_key(user_id: i64) -> String {
    format!("user:{user_id}")
}

async fn load_account_page_state(
    pool: &robominer_db::MySqlPool,
    user_id: i64,
    request: &Request,
) -> Result<AccountPageState, crate::page_context::PageLoadError> {
    let Some(current_user) = robominer_db::users::get_user_by_id(pool, user_id).await? else {
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

    if is_post(request) {
        let current_password = request
            .form
            .get("currentpassword")
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

        if let Some(mutation) =
            apply_account_mutations(pool, user_id, request, password_verified).await?
        {
            message = mutation.message;
            error_message = mutation.error_message;
            reissue_session_version = mutation.reissue_session_version;
            if let Some(submitted_username) = mutation.submitted_username {
                username = submitted_username;
            }
            if let Some(submitted_email) = mutation.submitted_email {
                email = submitted_email;
            }
            if message.is_some()
                && let Some(updated_user) =
                    robominer_db::users::get_user_by_id(pool, user_id).await?
            {
                username = updated_user.username;
                email = updated_user.email;
                current_username = username.clone();
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
