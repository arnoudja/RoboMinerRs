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
        reject_invalid_anonymous_csrf(&request("POST", HashMap::new(), Some(&cookie))).is_some()
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
