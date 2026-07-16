use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::http::Request;

type HmacSha256 = Hmac<Sha256>;

const SESSION_COOKIE_NAME: &str = "robominer_session";
pub const DEFAULT_SESSION_TTL_HOURS: u64 = 24;
pub const DEFAULT_SESSION_TTL_SECS: u64 = DEFAULT_SESSION_TTL_HOURS * 60 * 60;
const SESSION_REMEMBER_TTL_SECS: u64 = 30 * 24 * 60 * 60;
pub const DEFAULT_DEV_SESSION_SECRET: &str = "robominer-dev-session-secret-change-me";

static SESSION_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
static SECURE_COOKIES: AtomicBool = AtomicBool::new(false);
static SESSION_TTL_SECS: AtomicU64 = AtomicU64::new(DEFAULT_SESSION_TTL_SECS);
static SESSION_NONCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SessionClaims {
    pub user_id: i64,
    pub expires_at: u64,
    pub nonce: u64,
}

pub fn is_local_bind_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "localhost" | "::1")
}

pub fn resolve_session_secret(
    configured: Option<&str>,
    bind_host: &str,
) -> Result<String, &'static str> {
    if let Some(secret) = configured
        .map(str::trim)
        .filter(|secret| !secret.is_empty())
    {
        return Ok(secret.to_string());
    }

    if is_local_bind_host(bind_host) {
        eprintln!(
            "warning: ROBOMINER_SESSION_SECRET is not set; using an insecure development default"
        );
        return Ok(DEFAULT_DEV_SESSION_SECRET.to_string());
    }

    Err(
        "ROBOMINER_SESSION_SECRET (or sessionsecret in config) is required when binding to a non-localhost address",
    )
}

pub fn configure_session_secret(secret: &str) {
    let secret = secret.trim();
    assert!(!secret.is_empty(), "session secret must not be empty");
    let _ = SESSION_SECRET.get_or_init(|| secret.as_bytes().to_vec());
}

pub fn configure_secure_cookies(enabled: bool) {
    SECURE_COOKIES.store(enabled, Ordering::Relaxed);
}

pub fn configure_session_ttl_secs(ttl_secs: u64) {
    SESSION_TTL_SECS.store(ttl_secs, Ordering::Relaxed);
}

pub fn resolve_session_ttl_secs(
    env_secs: Option<&str>,
    env_hours: Option<&str>,
    config_secs: Option<&str>,
    config_hours: Option<&str>,
) -> Result<u64, String> {
    if let Some(value) = env_secs {
        return parse_session_ttl_secs(value, "ROBOMINER_SESSION_TTL_SECS");
    }
    if let Some(value) = env_hours {
        return parse_session_ttl_hours(value, "ROBOMINER_SESSION_TTL_HOURS");
    }
    if let Some(value) = config_secs {
        return parse_session_ttl_secs(value, "sessionttlsecs");
    }
    if let Some(value) = config_hours {
        return parse_session_ttl_hours(value, "sessionttlhours");
    }
    Ok(DEFAULT_SESSION_TTL_SECS)
}

fn parse_session_ttl_secs(value: &str, name: &str) -> Result<u64, String> {
    let ttl_secs = value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    validate_session_ttl_secs(ttl_secs, name)
}

fn parse_session_ttl_hours(value: &str, name: &str) -> Result<u64, String> {
    let hours = value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a positive integer"))?;
    if hours == 0 {
        return Err(format!("{name} must be greater than 0"));
    }
    let ttl_secs = hours
        .checked_mul(60 * 60)
        .ok_or_else(|| format!("{name} is too large"))?;
    validate_session_ttl_secs(ttl_secs, name)
}

fn validate_session_ttl_secs(ttl_secs: u64, name: &str) -> Result<u64, String> {
    if ttl_secs == 0 {
        return Err(format!("{name} must be greater than 0"));
    }
    Ok(ttl_secs)
}

pub(crate) fn secure_cookie_suffix() -> &'static str {
    if SECURE_COOKIES.load(Ordering::Relaxed) {
        "; Secure"
    } else {
        ""
    }
}

pub(crate) fn user_id_from_request(request: &Request) -> Option<i64> {
    session_from_request(request).map(|session| session.user_id)
}

pub(crate) fn session_from_request(request: &Request) -> Option<SessionClaims> {
    request
        .headers
        .get("cookie")
        .and_then(|cookies| cookie_value(cookies, SESSION_COOKIE_NAME))
        .and_then(|value| verify_session_token(&value))
}

pub(crate) fn session_from_cookie_header(cookies: &str) -> Option<SessionClaims> {
    cookie_value(cookies, SESSION_COOKIE_NAME).and_then(|value| verify_session_token(&value))
}

pub(crate) fn session_set_cookie_header(user_id: i64, persistent: bool) -> String {
    let ttl_secs = session_ttl_secs(persistent);
    let expires_at = session_expiry_timestamp(ttl_secs);
    let token = create_session_token(user_id, expires_at, new_session_nonce());
    format!(
        "{SESSION_COOKIE_NAME}={token}; Max-Age={ttl_secs}; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

/// Re-issue the session cookie for the given claims (used when rotating the CSRF nonce).
pub(crate) fn session_cookie_header_for_claims(session: SessionClaims) -> String {
    let max_age = session
        .expires_at
        .saturating_sub(current_unix_timestamp())
        .max(1);
    let token = create_session_token(session.user_id, session.expires_at, session.nonce);
    format!(
        "{SESSION_COOKIE_NAME}={token}; Max-Age={max_age}; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

pub(crate) fn new_session_nonce() -> u64 {
    SESSION_NONCE.fetch_add(1, Ordering::Relaxed)
}

fn session_ttl_secs(persistent: bool) -> u64 {
    if persistent {
        SESSION_REMEMBER_TTL_SECS
    } else {
        SESSION_TTL_SECS.load(Ordering::Relaxed)
    }
}

pub(crate) fn session_clear_cookie_header() -> String {
    format!(
        "{SESSION_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

pub(crate) fn format_authenticated_cookie(user_id: i64, username: &str) -> String {
    format!(
        "{}; robominer_username={}",
        session_set_cookie_header(user_id, false),
        cookie_encode(username)
    )
}

pub(crate) fn cookie_value(cookies: &str, name: &str) -> Option<String> {
    cookies.split(';').find_map(|cookie| {
        let (cookie_name, value) = cookie.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_string())
    })
}

fn create_session_token(user_id: i64, expires_at: u64, nonce: u64) -> String {
    let payload = format!("{user_id}.{expires_at}.{nonce}");
    let signature = sign_payload(&payload);
    format!("{payload}.{signature}")
}

fn verify_session_token(token: &str) -> Option<SessionClaims> {
    let (payload, signature) = token.rsplit_once('.')?;
    let expected_signature = sign_payload(payload);
    if !constant_time_eq(signature, &expected_signature) {
        return None;
    }

    let mut parts = payload.split('.');
    let user_id = parts.next()?.parse::<i64>().ok()?;
    let expires_at = parts.next()?.parse::<u64>().ok()?;
    let nonce = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if user_id <= 0 {
        return None;
    }
    if current_unix_timestamp() > expires_at {
        return None;
    }

    Some(SessionClaims {
        user_id,
        expires_at,
        nonce,
    })
}

fn sign_payload(payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(session_secret())
        .expect("session secret length should be valid for HMAC");
    mac.update(payload.as_bytes());
    encode_hex(&mac.finalize().into_bytes())
}

pub(crate) fn sign_csrf_session_payload(user_id: i64, nonce: u64) -> String {
    sign_payload(&format!("csrf.v2.{user_id}.{nonce}"))
}

pub(crate) fn sign_csrf_anon_payload(nonce: u64) -> String {
    sign_payload(&format!("csrf.anon.v1.{nonce}"))
}

pub(crate) fn constant_time_eq_str(left: &str, right: &str) -> bool {
    constant_time_eq(left, right)
}

fn session_secret() -> &'static [u8] {
    SESSION_SECRET.get_or_init(|| DEFAULT_DEV_SESSION_SECRET.as_bytes().to_vec())
}

fn session_expiry_timestamp(ttl_secs: u64) -> u64 {
    current_unix_timestamp().saturating_add(ttl_secs)
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.bytes()
        .zip(right.bytes())
        .fold(0u8, |acc, (left_byte, right_byte)| {
            acc | (left_byte ^ right_byte)
        })
        == 0
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn cookie_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'@' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
fn create_session_token_for_tests(user_id: i64) -> String {
    create_session_token(user_id, u64::MAX / 2, new_session_nonce())
}

#[cfg(test)]
mod tests {
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
            super::configure_session_secret("test-session-secret");
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
    fn resolve_session_secret_allows_dev_default_on_localhost() {
        let secret = resolve_session_secret(None, "127.0.0.1").expect("secret should resolve");
        assert_eq!(secret, super::DEFAULT_DEV_SESSION_SECRET);
    }

    #[test]
    fn resolve_session_secret_requires_secret_for_public_bind() {
        let error = resolve_session_secret(None, "0.0.0.0").unwrap_err();
        assert!(error.contains("ROBOMINER_SESSION_SECRET"));
    }

    #[test]
    fn valid_session_token_returns_user_id() {
        ensure_test_session_secret();
        let token = create_session_token_for_tests(42);
        let session = verify_session_token(&token).expect("valid session");
        assert_eq!(session.user_id, 42);
        assert!(session.nonce > 0);
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
        let token = super::create_session_token(42, 1, 1);
        assert_eq!(verify_session_token(&token), None);
    }

    #[test]
    fn user_id_from_request_uses_signed_session_cookie() {
        ensure_test_session_secret();
        let cookie = session_set_cookie_header(77, false);
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
    }

    #[test]
    fn session_set_cookie_header_uses_configured_max_age() {
        ensure_test_session_secret();
        super::configure_session_ttl_secs(3_600);
        let cookie = session_set_cookie_header(77, false);
        assert!(cookie.contains("; Max-Age=3600;"));
        super::configure_session_ttl_secs(DEFAULT_SESSION_TTL_SECS);
    }

    #[test]
    fn session_set_cookie_header_uses_default_max_age_matching_token_ttl() {
        ensure_test_session_secret();
        super::configure_session_ttl_secs(DEFAULT_SESSION_TTL_SECS);
        let cookie = session_set_cookie_header(77, false);
        assert!(cookie.starts_with("robominer_session="));
        assert!(cookie.contains("; Max-Age=86400;"));
    }

    #[test]
    fn persistent_session_set_cookie_header_uses_longer_max_age() {
        ensure_test_session_secret();
        let cookie = session_set_cookie_header(77, true);
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
        super::configure_session_secret("secure-cookie-test-secret");
        super::configure_secure_cookies(true);

        let cookie = session_set_cookie_header(42, false);

        assert!(cookie.ends_with("; Secure"));
    }
}
