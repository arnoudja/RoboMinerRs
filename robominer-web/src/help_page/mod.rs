use crate::{Request, Response, ServerConfig, help_pages, session_username};

pub(super) async fn help_page(request: &Request, config: &ServerConfig, show_welcome: bool) -> Response {
    let username = session_username(request);
    let hud = crate::app_shell::hud_markup(request, config).await;
    let welcome_banner = if show_welcome {
        help_pages::welcome_banner_markup()
    } else {
        ""
    };
    Response::html(help_pages::render_help_index(
        &username,
        hud.as_deref(),
        welcome_banner,
    ))
}

pub(super) async fn help_text_page(
    request: &Request,
    config: &ServerConfig,
    guide_href: &str,
    step: Option<i64>,
) -> Response {
    let username = session_username(request);
    let hud = crate::app_shell::hud_markup(request, config).await;
    match help_pages::render_help_article(&username, hud.as_deref(), guide_href, step) {
        Some(html) => Response::html(html),
        None => Response::not_found(),
    }
}

#[cfg(test)]
mod tests;
