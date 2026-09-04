use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::http::Request;

use config::{DEFAULT_DEV_SESSION_SECRET, session_ttl_secs};

mod config;

pub(crate) use config::secure_cookie_suffix;

type HmacSha256 = Hmac<Sha256>;

const SESSION_COOKIE_NAME: &str = "robominer_session";

static SESSION_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
static SESSION_NONCE: AtomicU64 = AtomicU64::new(1);

#[allow(unused_imports)]
pub use config::{
    DEFAULT_SESSION_TTL_HOURS, DEFAULT_SESSION_TTL_SECS, configure_secure_cookies,
    configure_session_ttl_secs, is_local_bind_host, resolve_secure_cookies, resolve_session_secret,
    resolve_session_ttl_secs, validate_trust_proxy_bind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SessionClaims {
    pub user_id: i64,
    pub expires_at: u64,
    pub nonce: u64,
    pub session_version: i32,
}

pub fn configure_session_secret(secret: &str) -> Result<(), String> {
    let secret = secret.trim();
    if secret.is_empty() {
        return Err("session secret must not be empty".to_string());
    }
    if secret.len() < config::MIN_SESSION_SECRET_LEN {
        return Err(format!(
            "session secret must be at least {} characters",
            config::MIN_SESSION_SECRET_LEN
        ));
    }
    if HmacSha256::new_from_slice(secret.as_bytes()).is_err() {
        return Err("session secret length is invalid for HMAC".to_string());
    }
    let _ = SESSION_SECRET.get_or_init(|| secret.as_bytes().to_vec());
    Ok(())
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

#[cfg(any(test, debug_assertions))]
pub(crate) fn session_from_cookie_header(cookies: &str) -> Option<SessionClaims> {
    cookie_value(cookies, SESSION_COOKIE_NAME).and_then(|value| verify_session_token(&value))
}

pub(crate) fn session_set_cookie_header(
    user_id: i64,
    persistent: bool,
    session_version: i32,
) -> String {
    let ttl_secs = session_ttl_secs(persistent);
    let expires_at = session_expiry_timestamp(ttl_secs);
    let token = create_session_token(user_id, expires_at, new_session_nonce(), session_version);
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
    let token = create_session_token(
        session.user_id,
        session.expires_at,
        session.nonce,
        session.session_version,
    );
    format!(
        "{SESSION_COOKIE_NAME}={token}; Max-Age={max_age}; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

pub(crate) fn new_session_nonce() -> u64 {
    SESSION_NONCE.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn session_clear_cookie_header() -> String {
    format!(
        "{SESSION_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

pub(crate) fn username_set_cookie_header(username: &str) -> String {
    format!(
        "robominer_username={}; Path=/; HttpOnly; SameSite=Lax{}",
        cookie_encode(username),
        secure_cookie_suffix()
    )
}

pub(crate) fn username_clear_cookie_header() -> String {
    format!(
        "robominer_username=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax{}",
        secure_cookie_suffix()
    )
}

#[cfg(any(test, debug_assertions))]
pub(crate) fn format_authenticated_cookie(user_id: i64, username: &str) -> String {
    format!(
        "{}; robominer_username={}",
        session_set_cookie_header(user_id, false, 0),
        cookie_encode(username)
    )
}

pub(crate) fn cookie_value(cookies: &str, name: &str) -> Option<String> {
    cookies.split(';').find_map(|cookie| {
        let (cookie_name, value) = cookie.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_string())
    })
}

fn create_session_token(user_id: i64, expires_at: u64, nonce: u64, session_version: i32) -> String {
    let payload = format!("{user_id}.{expires_at}.{nonce}.{session_version}");
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
    let session_version = match parts.next() {
        Some(value) => value.parse::<i32>().ok()?,
        None => 0,
    };
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
        session_version,
    })
}

fn sign_payload(payload: &str) -> String {
    let Ok(mut mac) = HmacSha256::new_from_slice(session_secret()) else {
        return String::new();
    };
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

pub(crate) fn cookie_encode(value: &str) -> String {
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
    create_session_token(user_id, u64::MAX / 2, new_session_nonce(), 0)
}

#[cfg(test)]
mod tests;
