use std::sync::atomic::{AtomicU64, Ordering};

use crate::http::{Request, Response};
use crate::request_helpers::is_post;
use crate::session::{self, cookie_value};

pub(crate) const CSRF_FIELD_NAME: &str = "csrfToken";
pub(crate) const ANON_CSRF_COOKIE_NAME: &str = "robominer_csrf";
const ANON_CSRF_COOKIE_MAX_AGE_SECS: u64 = 60 * 60;

static ANON_CSRF_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn csrf_token_for_user(user_id: i64) -> String {
    session::sign_csrf_payload(user_id)
}

pub(crate) fn valid_csrf_token(request: &Request, user_id: i64) -> bool {
    let Some(provided) = request.form.get(CSRF_FIELD_NAME) else {
        return false;
    };
    session::constant_time_eq_str(provided, &csrf_token_for_user(user_id))
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

pub(crate) fn html_with_csrf(user_id: i64, html: String) -> Response {
    Response::html(crate::html::inject_csrf_tokens(
        &html,
        &csrf_token_for_user(user_id),
    ))
}

/// Mint or reuse a double-submit CSRF cookie for anonymous login/signup pages.
pub(crate) fn html_with_anonymous_csrf(request: &Request, html: String) -> Response {
    let token = anonymous_csrf_token_for_response(request);
    Response::html(crate::html::inject_csrf_tokens(&html, &token))
        .with_header("Set-Cookie", anonymous_csrf_cookie_header(&token))
}

pub(crate) fn anonymous_csrf_token_for_response(request: &Request) -> String {
    if let Some(existing) = anonymous_csrf_cookie(request).filter(|token| looks_like_csrf_token(token))
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
    let Some(cookie_token) = anonymous_csrf_cookie(request).filter(|token| looks_like_csrf_token(token))
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
        ANON_CSRF_COOKIE_NAME, CSRF_FIELD_NAME, csrf_token_for_user, html_with_anonymous_csrf,
        new_anonymous_csrf_token, reject_invalid_anonymous_csrf, reject_invalid_csrf,
        valid_anonymous_csrf, valid_csrf_token,
    };
    use crate::Request;

    fn ensure_secret() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            crate::session::configure_session_secret("csrf-unit-test-secret");
        });
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
            path: "/login".to_string(),
            query: HashMap::new(),
            form,
            form_values,
            headers,
        }
    }

    #[test]
    fn csrf_token_is_stable_per_user_and_differs_across_users() {
        ensure_secret();
        let token_a = csrf_token_for_user(7);
        let token_b = csrf_token_for_user(7);
        let token_c = csrf_token_for_user(8);
        assert_eq!(token_a, token_b);
        assert_ne!(token_a, token_c);
        assert_eq!(token_a.len(), 64);
    }

    #[test]
    fn valid_csrf_token_accepts_matching_post_form_value() {
        ensure_secret();
        let mut form = HashMap::new();
        form.insert(CSRF_FIELD_NAME.to_string(), csrf_token_for_user(42));
        assert!(valid_csrf_token(&request("POST", form, None), 42));
    }

    #[test]
    fn valid_csrf_token_rejects_missing_wrong_or_get() {
        ensure_secret();
        assert!(!valid_csrf_token(
            &request("POST", HashMap::new(), None),
            42
        ));

        let mut wrong = HashMap::new();
        wrong.insert(CSRF_FIELD_NAME.to_string(), "deadbeef".to_string());
        assert!(!valid_csrf_token(&request("POST", wrong, None), 42));

        let mut form = HashMap::new();
        form.insert(CSRF_FIELD_NAME.to_string(), csrf_token_for_user(42));
        assert!(valid_csrf_token(&request("GET", form.clone(), None), 42));
        assert!(reject_invalid_csrf(&request("GET", form.clone(), None), 42).is_none());
        assert!(reject_invalid_csrf(&request("POST", HashMap::new(), None), 42).is_some());
        assert!(reject_invalid_csrf(&request("POST", form, None), 42).is_none());
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
        assert!(reject_invalid_anonymous_csrf(&request(
            "POST",
            HashMap::new(),
            Some(&cookie)
        ))
        .is_some());
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
