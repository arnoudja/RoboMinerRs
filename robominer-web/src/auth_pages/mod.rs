use crate::html::page_footer;
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

pub(super) fn logoff_page() -> Response {
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
        process::process_login_request(pool, request, config.allow_signup, config.trust_proxy).await;

    match result {
        Ok(response) => response,
        Err(error) => Response::service_unavailable(format!("Unable to process login: {error}")),
    }
}

mod process;
mod render;

#[cfg(test)]
mod tests;
