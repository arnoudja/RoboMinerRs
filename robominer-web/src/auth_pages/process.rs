use crate::rate_limit::{
    auth_attempt_is_rate_limited, client_ip, log_auth_failure, record_auth_attempt,
};
use crate::request_helpers::valid_login_return_to;
use crate::session::{self, cookie_value};
use crate::{Request, Response, is_post};

use super::LoginPageState;
use super::signup_pow;

pub(super) async fn process_login_request(
    pool: &robominer_db::MySqlPool,
    request: &Request,
    allow_signup: bool,
    trust_proxy: bool,
) -> Result<Response, crate::page_context::PageLoadError> {
    let return_to = return_to_from_request(request);
    let is_login_post = is_post(request)
        && (request.form.contains_key("loginName") || request.form.contains_key("password"));
    let is_signup_post = is_post(request)
        && (request.form.contains_key("newusername")
            || request.form.contains_key("email")
            || request.form.contains_key("newpassword")
            || request.form.contains_key("confirmpassword"));

    if (is_login_post || is_signup_post)
        && let Some(response) = crate::csrf::reject_invalid_anonymous_csrf(request)
    {
        return Ok(response);
    }

    if is_login_post {
        let login_name = request.form.get("loginName").cloned().unwrap_or_default();
        let password = request.form.get("password").cloned().unwrap_or_default();
        let ip = client_ip(request, trust_proxy);

        if auth_attempt_is_rate_limited(&ip, &login_name) {
            log_auth_failure(&ip, &login_name, "rate_limited");
            return Ok(Response::too_many_requests(
                "Too many login attempts. Please try again later.",
            ));
        }
        record_auth_attempt(&ip, &login_name);

        match robominer_db::verify_login(
            pool,
            robominer_db::VerifyLoginRequest {
                login_name: login_name.clone(),
                password,
            },
        )
        .await?
        .into_result()
        {
            Ok(verified) => {
                let username = robominer_db::users::get_user_by_id(pool, verified.user_id)
                    .await?
                    .map(|user| user.username)
                    .unwrap_or_else(|| login_name.clone());
                let redirect_target = return_to
                    .as_deref()
                    .and_then(valid_login_return_to)
                    .unwrap_or("miningQueue");
                let remember_login = request.form.contains_key("remember");
                return Ok(auth_redirect_response(
                    redirect_target,
                    verified.user_id,
                    verified.session_version,
                    &username,
                    remember_login,
                    remember_set_cookie_headers(&login_name, remember_login),
                ));
            }
            Err(_) => {
                log_auth_failure(&ip, &login_name, "invalid_credentials");
                crate::metrics::record_auth_failure();
                return Ok(login_html(
                    request,
                    &LoginPageState {
                        login_name,
                        new_username: String::new(),
                        email: String::new(),
                        error_message: Some(login_failure_message().to_string()),
                        show_signup: false,
                        allow_signup,
                        return_to,
                    },
                ));
            }
        }
    }

    if is_signup_post {
        let new_username = request.form.get("newusername").cloned().unwrap_or_default();
        let email = request.form.get("email").cloned().unwrap_or_default();
        let new_password = request.form.get("newpassword").cloned().unwrap_or_default();
        let confirm_password = request
            .form
            .get("confirmpassword")
            .cloned()
            .unwrap_or_default();
        let ip = client_ip(request, trust_proxy);

        if auth_attempt_is_rate_limited(&ip, &new_username) {
            log_auth_failure(&ip, &new_username, "rate_limited");
            return Ok(Response::too_many_requests(
                "Too many sign-up attempts. Please try again later.",
            ));
        }
        record_auth_attempt(&ip, &new_username);

        let error_message = if !allow_signup {
            Some(signup_disabled_message().to_string())
        } else if new_password != confirm_password {
            Some(signup_password_mismatch_message().to_string())
        } else if !signup_pow::verify_solution(
            request
                .form
                .get(crate::csrf::CSRF_FIELD_NAME)
                .map(String::as_str)
                .unwrap_or(""),
            request
                .form
                .get(signup_pow::POW_NONCE_FIELD)
                .map(String::as_str)
                .unwrap_or(""),
        ) {
            crate::metrics::record_signup_failure();
            Some(signup_pow_failed_message().to_string())
        } else {
            match robominer_db::create_user(
                pool,
                robominer_db::CreateUserRequest {
                    username: new_username.clone(),
                    email: email.clone(),
                    password: new_password,
                },
            )
            .await?
            .into_result()
            {
                Ok(created) => {
                    return Ok(auth_redirect_response(
                        "help?welcome=1",
                        created.user_id,
                        created.session_version,
                        &new_username,
                        false,
                        Vec::new(),
                    ));
                }
                Err(rejection) => {
                    log_auth_failure(&ip, &new_username, "signup_rejected");
                    crate::metrics::record_signup_failure();
                    Some(
                        robominer_domain::rejection_messages::create_user_rejection_player_message(
                            rejection,
                        )
                        .to_string(),
                    )
                }
            }
        };

        return Ok(login_html(
            request,
            &LoginPageState {
                login_name: String::new(),
                new_username,
                email,
                error_message,
                show_signup: allow_signup,
                allow_signup,
                return_to,
            },
        ));
    }

    Ok(login_html(
        request,
        &LoginPageState {
            login_name: request
                .headers
                .get("cookie")
                .and_then(|cookies| cookie_value(cookies, remember_cookie_name()))
                .unwrap_or_default(),
            new_username: String::new(),
            email: String::new(),
            error_message: None,
            show_signup: allow_signup && request.query.contains_key("signup"),
            allow_signup,
            return_to,
        },
    ))
}

fn login_html(request: &Request, state: &LoginPageState) -> Response {
    crate::csrf::html_with_anonymous_csrf(request, super::render::render_login_page(state))
}

fn return_to_from_request(request: &Request) -> Option<String> {
    request
        .query
        .get("returnTo")
        .or_else(|| request.form.get("returnTo"))
        .and_then(|value| valid_login_return_to(value))
        .map(str::to_string)
}

pub(super) fn auth_redirect_response(
    location: &str,
    user_id: i64,
    session_version: i32,
    username: &str,
    persistent_session: bool,
    remember_cookies: Vec<String>,
) -> Response {
    let mut response = Response::redirect(location)
        .with_header(
            "Set-Cookie",
            session::session_set_cookie_header(user_id, persistent_session, session_version),
        )
        .with_header("Set-Cookie", session::username_set_cookie_header(username));
    response = session::with_set_cookies(response, session::legacy_auth_cookie_clear_headers());
    if session::secure_cookies_enabled() {
        response =
            session::with_set_cookies(response, crate::csrf::anonymous_csrf_clear_cookie_headers());
    }
    response = session::with_set_cookies(response, remember_cookies);
    response
}

const LEGACY_REMEMBER_COOKIE_NAME: &str = "remember";
const HOST_REMEMBER_COOKIE_NAME: &str = "__Host-robominer_remember";

pub(super) fn remember_cookie_name() -> &'static str {
    if session::secure_cookies_enabled() {
        HOST_REMEMBER_COOKIE_NAME
    } else {
        LEGACY_REMEMBER_COOKIE_NAME
    }
}

/// Set-Cookie headers for remember (primary + legacy clear when Secure).
pub(super) fn remember_set_cookie_headers(login_name: &str, remember: bool) -> Vec<String> {
    let secure = session::secure_cookie_suffix();
    if remember {
        let mut headers = vec![format!(
            "{}={}; Max-Age=2678400; Path=/; HttpOnly; SameSite=Lax{secure}",
            remember_cookie_name(),
            session::cookie_encode(login_name)
        )];
        if session::secure_cookies_enabled() {
            headers.push(format!(
                "{LEGACY_REMEMBER_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{secure}"
            ));
        }
        headers
    } else {
        remember_clear_cookie_headers()
    }
}

pub(super) fn remember_clear_cookie_header() -> String {
    format!(
        "{}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
        remember_cookie_name(),
        session::secure_cookie_suffix()
    )
}

pub(super) fn remember_clear_cookie_headers() -> Vec<String> {
    let mut headers = vec![remember_clear_cookie_header()];
    if session::secure_cookies_enabled() {
        headers.push(format!(
            "{LEGACY_REMEMBER_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
            session::secure_cookie_suffix()
        ));
    }
    headers
}

pub(super) fn login_failure_message() -> &'static str {
    "Invalid login name or password."
}

pub(super) fn signup_password_mismatch_message() -> &'static str {
    "The passwords do not match."
}

fn signup_disabled_message() -> &'static str {
    "Sign up is not available on this server."
}

fn signup_pow_failed_message() -> &'static str {
    "Sign-up verification failed. Please try again."
}
