use std::sync::atomic::{AtomicU64, Ordering};

use crate::http::{Request, Response};
use crate::request_helpers::is_post;
use crate::session::{self, SessionClaims, cookie_value};

pub(crate) const CSRF_FIELD_NAME: &str = "csrfToken";
pub(crate) const LEGACY_ANON_CSRF_COOKIE_NAME: &str = "robominer_csrf";
pub(crate) const HOST_ANON_CSRF_COOKIE_NAME: &str = "__Host-robominer_csrf";
const ANON_CSRF_COOKIE_MAX_AGE_SECS: u64 = 60 * 60;

static ANON_CSRF_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Active anonymous CSRF cookie name: `__Host-*` when Secure is on, else unprefixed.
pub(crate) fn anon_csrf_cookie_name() -> &'static str {
    if session::secure_cookies_enabled() {
        HOST_ANON_CSRF_COOKIE_NAME
    } else {
        LEGACY_ANON_CSRF_COOKIE_NAME
    }
}

/// CSRF token bound to a session nonce (rotates when the session nonce changes).
pub fn csrf_token_for_session(user_id: i64, nonce: u64) -> String {
    session::sign_csrf_session_payload(user_id, nonce)
}

/// Derive the authenticated CSRF token from a Cookie header value.
#[cfg(any(test, debug_assertions))]
pub fn csrf_token_from_cookie(cookies: &str) -> Option<String> {
    session::session_from_cookie_header(cookies)
        .map(|session| csrf_token_for_session(session.user_id, session.nonce))
}

pub(crate) fn valid_csrf_token(request: &Request, user_id: i64) -> bool {
    let Some(provided) = request.form.get(CSRF_FIELD_NAME) else {
        return false;
    };
    let Some(session) = session::session_from_request(request) else {
        return false;
    };
    if session.user_id != user_id {
        return false;
    }
    session::constant_time_eq_str(
        provided,
        &csrf_token_for_session(session.user_id, session.nonce),
    )
}

/// Reject authenticated POST requests that omit or forge the CSRF token.
pub(crate) fn reject_invalid_csrf(request: &Request, user_id: i64) -> Option<Response> {
    if !is_post(request) {
        return None;
    }
    if valid_csrf_token(request, user_id) {
        None
    } else {
        Some(Response::forbidden("Invalid or missing CSRF token"))
    }
}

/// Inject CSRF tokens into HTML. After a successful authenticated POST, rotate the
/// session nonce and Set-Cookie so the next form uses a fresh token.
pub(crate) fn html_with_csrf(request: &Request, user_id: i64, html: String) -> Response {
    let Some(session) = session::session_from_request(request).filter(|s| s.user_id == user_id)
    else {
        return Response::html(html);
    };

    let (session, rotate_cookie) = if is_post(request) {
        let rotated = SessionClaims {
            nonce: session::new_session_nonce(),
            ..session
        };
        (rotated, true)
    } else {
        (session, false)
    };

    let mut response = Response::html(crate::html::inject_csrf_tokens(
        &html,
        &csrf_token_for_session(session.user_id, session.nonce),
    ));
    if rotate_cookie {
        response = response.with_header(
            "Set-Cookie",
            session::session_cookie_header_for_claims(session),
        );
    }
    response
}

/// Mint or reuse a double-submit CSRF cookie for anonymous login/signup pages.
pub(crate) fn html_with_anonymous_csrf(request: &Request, html: String) -> Response {
    let token = anonymous_csrf_token_for_response(request);
    let mut response = Response::html(crate::html::inject_csrf_tokens(&html, &token))
        .with_header("Set-Cookie", anonymous_csrf_cookie_header(&token));
    if session::secure_cookies_enabled() {
        response = response.with_header(
            "Set-Cookie",
            format!(
                "{LEGACY_ANON_CSRF_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
                session::secure_cookie_suffix()
            ),
        );
    }
    response
}

pub(crate) fn anonymous_csrf_token_for_response(request: &Request) -> String {
    if let Some(existing) =
        anonymous_csrf_cookie(request).filter(|token| looks_like_csrf_token(token))
    {
        existing
    } else {
        new_anonymous_csrf_token()
    }
}

pub(crate) fn new_anonymous_csrf_token() -> String {
    let counter = ANON_CSRF_COUNTER.fetch_add(1, Ordering::Relaxed);
    session::sign_csrf_anon_payload(counter)
}

pub(crate) fn anonymous_csrf_cookie_header(token: &str) -> String {
    format!(
        "{}={token}; Max-Age={ANON_CSRF_COOKIE_MAX_AGE_SECS}; Path=/; HttpOnly; SameSite=Lax{}",
        anon_csrf_cookie_name(),
        session::secure_cookie_suffix()
    )
}

pub(crate) fn anonymous_csrf_clear_cookie_header() -> String {
    format!(
        "{}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
        anon_csrf_cookie_name(),
        session::secure_cookie_suffix()
    )
}

/// Expire the active anon CSRF cookie and, when using `__Host-*`, the legacy name too.
pub(crate) fn anonymous_csrf_clear_cookie_headers() -> Vec<String> {
    let mut headers = vec![anonymous_csrf_clear_cookie_header()];
    if session::secure_cookies_enabled() {
        headers.push(format!(
            "{LEGACY_ANON_CSRF_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
            session::secure_cookie_suffix()
        ));
    }
    headers
}

pub(crate) fn anonymous_csrf_cookie(request: &Request) -> Option<String> {
    request
        .headers
        .get("cookie")
        .and_then(|cookies| cookie_value(cookies, anon_csrf_cookie_name()))
}

pub(crate) fn valid_anonymous_csrf(request: &Request) -> bool {
    let Some(cookie_token) =
        anonymous_csrf_cookie(request).filter(|token| looks_like_csrf_token(token))
    else {
        return false;
    };
    let Some(form_token) = request.form.get(CSRF_FIELD_NAME) else {
        return false;
    };
    session::constant_time_eq_str(&cookie_token, form_token)
}

/// Reject login/signup POST requests that omit or forge the double-submit CSRF token.
pub(crate) fn reject_invalid_anonymous_csrf(request: &Request) -> Option<Response> {
    if !is_post(request) {
        return None;
    }
    if valid_anonymous_csrf(request) {
        None
    } else {
        Some(Response::forbidden("Invalid or missing CSRF token"))
    }
}

fn looks_like_csrf_token(token: &str) -> bool {
    token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
