//! HTTP routing: session gate, legacy redirects, and page dispatch.

mod dispatch;
mod redirect;
mod session_gate;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::Request;
use crate::{Response, ServerConfig, health};

use redirect::canonical_path_redirect;
use session_gate::{SessionStrip, clear_stale_session_cookies, strip_stale_session_cookie};

pub async fn route(request: &Request, config: &ServerConfig) -> Response {
    if matches!(request.path.as_str(), "/health" | "/Health")
        && matches!(request.method.as_str(), "GET" | "HEAD")
    {
        return health::health_response(config).await;
    }

    if let Some(response) = canonical_path_redirect(request) {
        return response;
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
