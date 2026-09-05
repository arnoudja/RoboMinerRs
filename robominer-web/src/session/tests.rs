use std::collections::HashMap;
use std::sync::Once;

use super::{
    DEFAULT_SESSION_TTL_SECS, SESSION_COOKIE_NAME, create_session_token_for_tests,
    format_authenticated_cookie, is_local_bind_host, resolve_session_secret,
    resolve_session_ttl_secs, session_clear_cookie_header, session_set_cookie_header,
    user_id_from_request, verify_session_token,
};
use crate::Request;

fn ensure_test_session_secret() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        super::configure_session_secret("test-session-secret-at-least-32-chars")
            .expect("configure secret");
        super::configure_secure_cookies(false);
    });
}

fn request_with_cookie(cookie: &str) -> Request {
    Request {
        method: "GET".to_string(),
        path: "/account".to_string(),
        query: HashMap::new(),
        form: HashMap::new(),
        form_values: HashMap::new(),
        headers: HashMap::from([("cookie".to_string(), cookie.to_string())]),
    }
}

#[test]
fn local_bind_hosts_are_detected() {
    assert!(is_local_bind_host("127.0.0.1"));
    assert!(is_local_bind_host("localhost"));
    assert!(is_local_bind_host("::1"));
    assert!(!is_local_bind_host("0.0.0.0"));
}

#[test]
fn resolve_session_secret_allows_dev_default_on_localhost_when_opted_in() {
    let secret = resolve_session_secret(None, "127.0.0.1", true).expect("secret should resolve");
    assert_eq!(secret, super::DEFAULT_DEV_SESSION_SECRET);
}

#[test]
fn resolve_session_secret_requires_opt_in_for_localhost_default() {
    let error = resolve_session_secret(None, "127.0.0.1", false).unwrap_err();
    assert!(error.contains("ROBOMINER_ALLOW_INSECURE_DEV_SECRET"));
}

#[test]
fn resolve_session_secret_requires_secret_for_public_bind() {
    let error = resolve_session_secret(None, "0.0.0.0", true).unwrap_err();
    assert!(error.contains("ROBOMINER_SESSION_SECRET"));
}

#[test]
fn resolve_session_secret_rejects_short_configured_secret() {
    let error = resolve_session_secret(Some("too-short"), "127.0.0.1", true).unwrap_err();
    assert!(error.contains("at least 32"));
}

#[test]
fn validate_trust_proxy_bind_requires_loopback() {
    assert!(super::validate_trust_proxy_bind("127.0.0.1", true).is_ok());
    assert!(super::validate_trust_proxy_bind("0.0.0.0", true).is_err());
    assert!(super::validate_trust_proxy_bind("0.0.0.0", false).is_ok());
}

#[test]
fn resolve_secure_cookies_defaults_off_on_loopback() {
    assert!(!super::resolve_secure_cookies(None, "127.0.0.1", false).unwrap());
    assert!(super::resolve_secure_cookies(Some(true), "127.0.0.1", false).unwrap());
    assert!(super::resolve_secure_cookies(Some(true), "127.0.0.1", true).unwrap());
}

#[test]
fn resolve_secure_cookies_requires_secure_on_non_loopback_bind() {
    let error = super::resolve_secure_cookies(None, "0.0.0.0", false).unwrap_err();
    assert!(error.contains("securecookies") || error.contains("ROBOMINER_SECURE_COOKIES"));
    let error = super::resolve_secure_cookies(Some(false), "0.0.0.0", false).unwrap_err();
    assert!(error.contains("securecookies") || error.contains("ROBOMINER_SECURE_COOKIES"));
    assert!(super::resolve_secure_cookies(Some(true), "0.0.0.0", false).unwrap());
}

#[test]
fn resolve_secure_cookies_requires_secure_when_trust_proxy() {
    let error = super::resolve_secure_cookies(None, "127.0.0.1", true).unwrap_err();
    assert!(error.contains("securecookies"));
    let error = super::resolve_secure_cookies(Some(false), "127.0.0.1", true).unwrap_err();
    assert!(error.contains("ROBOMINER_SECURE_COOKIES"));
}

#[test]
fn valid_session_token_returns_user_id() {
    ensure_test_session_secret();
    let token = create_session_token_for_tests(42);
    let session = verify_session_token(&token).expect("valid session");
    assert_eq!(session.user_id, 42);
    assert!(session.nonce > 0);
    assert_eq!(session.session_version, 0);
}

#[test]
fn session_token_embeds_and_verifies_session_version() {
    ensure_test_session_secret();
    let token = super::create_session_token(42, u64::MAX / 2, 9, 7);
    let session = verify_session_token(&token).expect("valid session");
    assert_eq!(session.user_id, 42);
    assert_eq!(session.nonce, 9);
    assert_eq!(session.session_version, 7);
}

#[test]
fn session_token_without_version_field_is_rejected() {
    ensure_test_session_secret();
    // Legacy shape: userId.expiresAt.nonce.signature (no session_version).
    let payload = format!("{}.{}.{}", 42, u64::MAX / 2, 9);
    let signature = super::sign_payload(&payload);
    let legacy_token = format!("{payload}.{signature}");
    assert_eq!(
        verify_session_token(&legacy_token),
        None,
        "unversioned session tokens must be rejected after sunset"
    );
}

#[test]
fn tampered_session_token_is_rejected() {
    ensure_test_session_secret();
    let token = create_session_token_for_tests(42);
    let tampered = token.replacen("42", "99", 1);
    assert_eq!(verify_session_token(&tampered), None);
}

#[test]
fn expired_session_token_is_rejected() {
    ensure_test_session_secret();
    let token = super::create_session_token(42, 1, 1, 0);
    assert_eq!(verify_session_token(&token), None);
}

#[test]
fn user_id_from_request_uses_signed_session_cookie() {
    ensure_test_session_secret();
    let cookie = session_set_cookie_header(77, false, 0);
    let request = request_with_cookie(&cookie);

    assert_eq!(user_id_from_request(&request), Some(77));
}

#[test]
fn resolve_session_ttl_secs_defaults_to_twenty_four_hours() {
    assert_eq!(
        resolve_session_ttl_secs(None, None, None, None).expect("default ttl"),
        DEFAULT_SESSION_TTL_SECS
    );
}

#[test]
fn resolve_session_ttl_secs_prefers_env_over_config() {
    assert_eq!(
        resolve_session_ttl_secs(None, Some("48"), Some("3600"), None).expect("env hours"),
        48 * 60 * 60
    );
    assert_eq!(
        resolve_session_ttl_secs(Some("7200"), None, None, Some("12")).expect("env secs"),
        7200
    );
}

#[test]
fn resolve_session_ttl_secs_rejects_invalid_values() {
    assert!(resolve_session_ttl_secs(Some("0"), None, None, None).is_err());
    assert!(resolve_session_ttl_secs(None, Some("abc"), None, None).is_err());
    let over_max = (super::MAX_SESSION_TTL_SECS + 1).to_string();
    let error = resolve_session_ttl_secs(Some(&over_max), None, None, None).unwrap_err();
    assert!(error.contains("30 days"));
}

#[test]
fn session_set_cookie_header_uses_configured_max_age() {
    ensure_test_session_secret();
    super::configure_session_ttl_secs(3_600);
    let cookie = session_set_cookie_header(77, false, 0);
    assert!(cookie.contains("; Max-Age=3600;"));
    super::configure_session_ttl_secs(DEFAULT_SESSION_TTL_SECS);
}

#[test]
fn session_set_cookie_header_uses_default_max_age_matching_token_ttl() {
    ensure_test_session_secret();
    super::configure_session_ttl_secs(DEFAULT_SESSION_TTL_SECS);
    let cookie = session_set_cookie_header(77, false, 0);
    assert!(cookie.starts_with("robominer_session="));
    assert!(cookie.contains("; Max-Age=86400;"));
}

#[test]
fn persistent_session_set_cookie_header_uses_longer_max_age() {
    ensure_test_session_secret();
    let cookie = session_set_cookie_header(77, true, 0);
    assert!(cookie.contains("; Max-Age=2592000;"));
}

#[test]
fn user_id_from_request_ignores_query_parameter() {
    ensure_test_session_secret();
    let mut request = request_with_cookie("robominer_username=Player");
    request.query.insert("userId".to_string(), "42".to_string());

    assert_eq!(user_id_from_request(&request), None);
}

#[test]
fn user_id_from_request_ignores_legacy_user_id_cookie() {
    ensure_test_session_secret();
    let request = request_with_cookie("robominer_user_id=42");

    assert_eq!(user_id_from_request(&request), None);
}

#[test]
fn authenticated_cookie_helper_sets_session_and_username() {
    ensure_test_session_secret();
    let cookie = format_authenticated_cookie(42, "Player");

    assert!(cookie.contains(&format!("{SESSION_COOKIE_NAME}=")));
    assert!(cookie.contains("robominer_username=Player"));
    assert_eq!(
        user_id_from_request(&request_with_cookie(&cookie)),
        Some(42)
    );
}

#[test]
fn session_clear_cookie_expires_session() {
    assert!(session_clear_cookie_header().starts_with("robominer_session=; Max-Age=0;"));
}

#[test]
fn secure_cookie_suffix_is_applied_when_enabled() {
    super::configure_session_secret("secure-cookie-test-secret-32chars!!")
        .expect("configure secret");
    super::configure_secure_cookies(true);

    let cookie = session_set_cookie_header(42, false, 0);

    assert!(cookie.ends_with("; Secure"));
}
