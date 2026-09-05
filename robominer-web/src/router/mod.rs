//! HTTP routing: session gate and page dispatch.

mod dispatch;
mod route_policy;
mod session_gate;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::Request;
use crate::{Response, ServerConfig, health};

use session_gate::{SessionStrip, clear_stale_session_cookies, strip_stale_session_cookie};

async fn root_redirect(request: &Request) -> Response {
    if crate::request_helpers::request_user_id(request).is_some() {
        crate::Response::redirect(crate::routes::AppRoute::MiningQueue.href())
    } else {
        crate::request_helpers::login_redirect(request)
    }
}

pub async fn route(request: &Request, config: &ServerConfig) -> Response {
    if request.path == "/health" && matches!(request.method.as_str(), "GET" | "HEAD") {
        return health::health_response(config).await;
    }

    let (session_strip, effective_request) = match config.database_pool.as_ref() {
        Some(pool) if crate::session::session_from_request(request).is_some() => {
            let mut owned = request.clone();
            let strip = strip_stale_session_cookie(&mut owned, pool).await;
            (strip, Cow::Owned(owned))
        }
        Some(_) => (SessionStrip::Keep, Cow::Borrowed(request)),
        None => (SessionStrip::Keep, Cow::Borrowed(request)),
    };

    let mut response = dispatch::dispatch(effective_request.as_ref(), config).await;
    if matches!(session_strip, SessionStrip::InvalidatePermanently) {
        response = clear_stale_session_cookies(response);
    }
    response
}
