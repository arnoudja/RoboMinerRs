use crate::html::page_footer;
use crate::request_helpers::{is_post, request_user_id};
use crate::session;
use crate::static_assets::{PageStylesheet, robominer_stylesheet_tags};
use crate::{Request, Response, ServerConfig};

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

pub(super) fn logoff_page(request: &Request) -> Response {
    if is_post(request) {
        if let Some(user_id) = request_user_id(request)
            && let Some(response) = crate::csrf::reject_invalid_csrf(request, user_id)
        {
            return response;
        }
        return logoff_response_clearing_cookies();
    }

    // GET must not clear cookies (logout CSRF). Show the page only.
    logoff_html_response()
}

fn logoff_response_clearing_cookies() -> Response {
    logoff_html_response()
        .with_header("Set-Cookie", session::session_clear_cookie_header())
        .with_header(
            "Set-Cookie",
            "robominer_user_id=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax",
        )
        .with_header(
            "Set-Cookie",
            "robominer_username=; Max-Age=0; Path=/; SameSite=Lax",
        )
}

fn logoff_html_response() -> Response {
    Response::html(format!(
        r##"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
        {}
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
        robominer_stylesheet_tags(&[PageStylesheet::Auth]),
        render::render_logoff_body(),
        page_footer()
    ))
}

pub(super) async fn login_page(request: &Request, config: &ServerConfig) -> Response {
    let Some(pool) = config.database_pool.as_ref() else {
        return Response::service_unavailable(
            "Login requires ROBOMINER_DATABASE_URL to be configured",
        );
    };

    let result =
        process::process_login_request(pool, request, config.allow_signup, config.trust_proxy)
            .await;

    match result {
        Ok(response) => response,
        Err(error) => Response::service_unavailable(format!("Unable to process login: {error}")),
    }
}

mod process;
mod render;

#[cfg(test)]
mod tests;
