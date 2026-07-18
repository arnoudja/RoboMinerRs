use std::sync::atomic::{AtomicU64, Ordering};

use crate::http::{Request, Response};
use crate::request_helpers::is_post;
use crate::session::{self, SessionClaims, cookie_value};

pub(crate) const CSRF_FIELD_NAME: &str = "csrfToken";
pub(crate) const ANON_CSRF_COOKIE_NAME: &str = "robominer_csrf";
const ANON_CSRF_COOKIE_MAX_AGE_SECS: u64 = 60 * 60;

static ANON_CSRF_COUNTER: AtomicU64 = AtomicU64::new(1);

/// CSRF token bound to a session nonce (rotates when the session nonce changes).
pub fn csrf_token_for_session(user_id: i64, nonce: u64) -> String {
    session::sign_csrf_session_payload(user_id, nonce)
}

/// Derive the authenticated CSRF token from a Cookie header value.
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
    Response::html(crate::html::inject_csrf_tokens(&html, &token))
        .with_header("Set-Cookie", anonymous_csrf_cookie_header(&token))
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
        "{ANON_CSRF_COOKIE_NAME}={token}; Max-Age={ANON_CSRF_COOKIE_MAX_AGE_SECS}; Path=/; HttpOnly; SameSite=Lax{}",
        session::secure_cookie_suffix()
    )
}

pub(crate) fn anonymous_csrf_cookie(request: &Request) -> Option<String> {
    request
        .headers
        .get("cookie")
        .and_then(|cookies| cookie_value(cookies, ANON_CSRF_COOKIE_NAME))
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
mod tests {
    use std::collections::HashMap;
    use std::sync::Once;

    use super::{
        ANON_CSRF_COOKIE_NAME, CSRF_FIELD_NAME, csrf_token_for_session, csrf_token_from_cookie,
        html_with_anonymous_csrf, html_with_csrf, new_anonymous_csrf_token,
        reject_invalid_anonymous_csrf, reject_invalid_csrf, valid_anonymous_csrf, valid_csrf_token,
    };
    use crate::Request;
    use crate::session::{self, session_from_cookie_header};

    fn ensure_secret() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            crate::session::configure_session_secret("csrf-unit-test-secret");
        });
    }

    fn authenticated_cookie(user_id: i64) -> String {
        ensure_secret();
        session::session_set_cookie_header(user_id, false, 0)
    }

    fn request(method: &str, form: HashMap<String, String>, cookie: Option<&str>) -> Request {
        let form_values = form
            .iter()
            .map(|(name, value)| (name.clone(), vec![value.clone()]))
            .collect();
        let mut headers = HashMap::new();
        if let Some(cookie) = cookie {
            headers.insert("cookie".to_string(), cookie.to_string());
        }
        Request {
            method: method.to_string(),
            path: "/shop".to_string(),
            query: HashMap::new(),
            form,
            form_values,
            headers,
        }
    }

    #[test]
    fn csrf_token_is_bound_to_session_nonce() {
        ensure_secret();
        let cookie_a = authenticated_cookie(7);
        let cookie_b = authenticated_cookie(7);
        let token_a = csrf_token_from_cookie(&cookie_a).expect("token");
        let token_b = csrf_token_from_cookie(&cookie_b).expect("token");
        assert_ne!(
            token_a, token_b,
            "new sessions should mint distinct CSRF tokens"
        );
        assert_eq!(token_a.len(), 64);

        let session = session_from_cookie_header(&cookie_a).expect("session");
        assert_eq!(
            csrf_token_for_session(session.user_id, session.nonce),
            token_a
        );
        assert_ne!(
            csrf_token_for_session(session.user_id, session.nonce.wrapping_add(1)),
            token_a
        );
    }

    #[test]
    fn valid_csrf_token_accepts_matching_post_form_value() {
        let cookie = authenticated_cookie(42);
        let token = csrf_token_from_cookie(&cookie).expect("token");
        let mut form = HashMap::new();
        form.insert(CSRF_FIELD_NAME.to_string(), token);
        assert!(valid_csrf_token(&request("POST", form, Some(&cookie)), 42));
    }

    #[test]
    fn valid_csrf_token_rejects_missing_wrong_or_mismatched_session() {
        let cookie = authenticated_cookie(42);
        assert!(!valid_csrf_token(
            &request("POST", HashMap::new(), Some(&cookie)),
            42
        ));

        let mut wrong = HashMap::new();
        wrong.insert(CSRF_FIELD_NAME.to_string(), "deadbeef".to_string());
        assert!(!valid_csrf_token(
            &request("POST", wrong, Some(&cookie)),
            42
        ));

        let token = csrf_token_from_cookie(&cookie).expect("token");
        let mut form = HashMap::new();
        form.insert(CSRF_FIELD_NAME.to_string(), token);
        assert!(valid_csrf_token(
            &request("GET", form.clone(), Some(&cookie)),
            42
        ));
        assert!(reject_invalid_csrf(&request("GET", form.clone(), Some(&cookie)), 42).is_none());
        assert!(reject_invalid_csrf(&request("POST", HashMap::new(), Some(&cookie)), 42).is_some());
        assert!(reject_invalid_csrf(&request("POST", form, Some(&cookie)), 42).is_none());
    }

    #[test]
    fn html_with_csrf_rotates_session_nonce_after_post() {
        let cookie = authenticated_cookie(9);
        let before = session_from_cookie_header(&cookie).expect("session");
        let html =
            r#"<!DOCTYPE html><html><head></head><body><form method="post"></form></body></html>"#;
        let response = html_with_csrf(
            &request("POST", HashMap::new(), Some(&cookie)),
            9,
            html.into(),
        );
        let set_cookie = response
            .headers
            .iter()
            .find(|(name, _)| *name == "Set-Cookie")
            .map(|(_, value)| value.clone())
            .expect("POST HTML should rotate session cookie");
        let after = session_from_cookie_header(&set_cookie).expect("rotated session");
        assert_eq!(after.user_id, before.user_id);
        assert_eq!(after.expires_at, before.expires_at);
        assert_ne!(after.nonce, before.nonce);

        let body = String::from_utf8(response.body).expect("utf8");
        assert!(body.contains(&csrf_token_for_session(after.user_id, after.nonce)));
    }

    #[test]
    fn html_with_csrf_keeps_nonce_on_get() {
        let cookie = authenticated_cookie(9);
        let before = session_from_cookie_header(&cookie).expect("session");
        let html =
            r#"<!DOCTYPE html><html><head></head><body><form method="post"></form></body></html>"#;
        let response = html_with_csrf(
            &request("GET", HashMap::new(), Some(&cookie)),
            9,
            html.into(),
        );
        assert!(
            response
                .headers
                .iter()
                .all(|(name, _)| *name != "Set-Cookie")
        );
        let body = String::from_utf8(response.body).expect("utf8");
        assert!(body.contains(&csrf_token_for_session(before.user_id, before.nonce)));
    }

    #[test]
    fn anonymous_double_submit_csrf_requires_matching_cookie_and_form() {
        ensure_secret();
        let token = new_anonymous_csrf_token();
        let cookie = format!("{ANON_CSRF_COOKIE_NAME}={token}");
        let mut form = HashMap::new();
        form.insert(CSRF_FIELD_NAME.to_string(), token.clone());
        assert!(valid_anonymous_csrf(&request(
            "POST",
            form.clone(),
            Some(&cookie)
        )));
        assert!(reject_invalid_anonymous_csrf(&request("POST", form, Some(&cookie))).is_none());
        assert!(
            reject_invalid_anonymous_csrf(&request("POST", HashMap::new(), Some(&cookie)))
                .is_some()
        );
    }

    #[test]
    fn html_with_anonymous_csrf_sets_cookie_and_injects_form_field() {
        ensure_secret();
        let html = r#"<!DOCTYPE html><html><head></head><body><form method="post" action="Login"></form></body></html>"#;
        let response = html_with_anonymous_csrf(&request("GET", HashMap::new(), None), html.into());
        let body = String::from_utf8(response.body.clone()).expect("utf8");
        assert!(body.contains(r#"name="csrfToken""#));
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Set-Cookie"
                    && value.starts_with(&format!("{ANON_CSRF_COOKIE_NAME}=")))
        );
    }
}
