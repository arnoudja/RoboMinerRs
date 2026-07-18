use crate::html::page_footer;
use crate::rate_limit::{
    auth_attempt_is_rate_limited, client_ip, log_auth_failure, record_auth_attempt,
};
use crate::request_helpers::valid_login_return_to;
use crate::session::{self, cookie_value};
use crate::{Request, Response, ServerConfig, is_post};

#[derive(Debug)]
pub(super) struct LoginPageState {
    pub(super) login_name: String,
    pub(super) new_username: String,
    pub(super) email: String,
    pub(super) error_message: Option<String>,
    pub(super) show_signup: bool,
    pub(super) allow_signup: bool,
    pub(super) return_to: Option<String>,
}

pub(super) fn logoff_page() -> Response {
    Response::html(format!(
        r##"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
        <link rel="stylesheet" type="text/css" href="css/robominer.css">
        <title>RoboMiner - Logged off</title>
    </head>
    <body>
        <div class="main">
            <div class="interface">
                {}
            </div>
            {}
        </div>
    </body>
</html>"##,
        render::render_logoff_body(),
        page_footer()
    ))
    .with_header("Set-Cookie", session::session_clear_cookie_header())
    .with_header(
        "Set-Cookie",
        "robominer_user_id=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax",
    )
    .with_header(
        "Set-Cookie",
        "robominer_username=; Max-Age=0; Path=/; SameSite=Lax",
    )
    .with_header("Set-Cookie", "JSESSIONID=; Max-Age=0; Path=/; HttpOnly")
}

pub(super) async fn login_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Login requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let result =
        process_login_request(pool, request, config.allow_signup, config.trust_proxy).await;

    match result {
        Ok(response) => response,
        Err(error) => Response::service_unavailable(format!("Unable to process login: {error}")),
    }
}

async fn process_login_request(
    pool: &robominer_db::MySqlPool,
    request: &Request,
    allow_signup: bool,
    trust_proxy: bool,
) -> Result<Response, robominer_domain::DomainError> {
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
        {
            Ok(verified) => {
                let username = robominer_db::get_user_by_id(pool, verified.user_id)
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
                    remember_cookie(&login_name, remember_login),
                ));
            }
            Err(_) => {
                log_auth_failure(&ip, &login_name, "invalid_credentials");
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
            {
                Ok(created) => {
                    return Ok(auth_redirect_response(
                        "help?welcome=1",
                        created.user_id,
                        created.session_version,
                        &new_username,
                        false,
                        None,
                    ));
                }
                Err(rejection) => {
                    log_auth_failure(&ip, &new_username, "signup_rejected");
                    Some(create_user_rejection_message(rejection).to_string())
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
                .and_then(|cookies| cookie_value(cookies, "remember"))
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
    crate::csrf::html_with_anonymous_csrf(request, render::render_login_page(state))
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
    remember_cookie: Option<String>,
) -> Response {
    let mut response = Response::redirect(location)
        .with_header(
            "Set-Cookie",
            session::session_set_cookie_header(user_id, persistent_session, session_version),
        )
        .with_header(
            "Set-Cookie",
            format!(
                "robominer_username={}; Path=/; SameSite=Lax{}",
                cookie_encode(username),
                session::secure_cookie_suffix()
            ),
        );
    if let Some(cookie) = remember_cookie {
        response = response.with_header("Set-Cookie", cookie);
    }
    response
}

pub(super) fn remember_cookie(login_name: &str, remember: bool) -> Option<String> {
    let secure = session::secure_cookie_suffix();
    if remember {
        Some(format!(
            "remember={}; Max-Age=2678400; Path=/; SameSite=Lax{secure}",
            cookie_encode(login_name)
        ))
    } else {
        Some(format!(
            "remember=; Max-Age=0; Path=/; SameSite=Lax{secure}"
        ))
    }
}

fn cookie_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'@' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
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

pub(super) fn create_user_rejection_message(
    rejection: robominer_db::CreateUserRejection,
) -> &'static str {
    robominer_domain::create_user_rejection_player_message(rejection)
}

mod render;

#[cfg(test)]
mod tests;
