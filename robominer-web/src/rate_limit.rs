use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::http::Request;

const AUTH_WINDOW: Duration = Duration::from_secs(60);
const MAX_ATTEMPTS_PER_IP: usize = 10;
const MAX_ATTEMPTS_PER_LOGIN: usize = 5;

static AUTH_RATE_LIMITER: OnceLock<Mutex<AuthRateLimiter>> = OnceLock::new();

#[derive(Debug, Default)]
struct AuthRateLimiter {
    by_ip: HashMap<String, VecDeque<Instant>>,
    by_login: HashMap<String, VecDeque<Instant>>,
}

fn auth_rate_limiter() -> &'static Mutex<AuthRateLimiter> {
    AUTH_RATE_LIMITER.get_or_init(|| Mutex::new(AuthRateLimiter::default()))
}

impl AuthRateLimiter {
    fn prune(window: &mut VecDeque<Instant>, now: Instant) {
        while window
            .front()
            .is_some_and(|instant| now.duration_since(*instant) > AUTH_WINDOW)
        {
            window.pop_front();
        }
    }

    fn is_limited(&mut self, ip: &str, login_key: &str, now: Instant) -> bool {
        let ip_window = self.by_ip.entry(ip.to_string()).or_default();
        Self::prune(ip_window, now);
        if ip_window.len() >= MAX_ATTEMPTS_PER_IP {
            return true;
        }

        if login_key.is_empty() {
            return false;
        }
        let login_window = self.by_login.entry(login_key.to_string()).or_default();
        Self::prune(login_window, now);
        login_window.len() >= MAX_ATTEMPTS_PER_LOGIN
    }

    fn record(&mut self, ip: &str, login_key: &str, now: Instant) {
        let ip_window = self.by_ip.entry(ip.to_string()).or_default();
        Self::prune(ip_window, now);
        ip_window.push_back(now);

        if login_key.is_empty() {
            return;
        }
        let login_window = self.by_login.entry(login_key.to_string()).or_default();
        Self::prune(login_window, now);
        login_window.push_back(now);
    }
}

/// Client IP for rate limiting / auth logs.
/// Prefers proxy headers, then peer address injected by the Axum acceptor.
pub(crate) fn client_ip(request: &Request) -> String {
    if let Some(forwarded) = request.headers.get("x-forwarded-for")
        && let Some(first) = forwarded.split(',').next()
    {
        let trimmed = first.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(real_ip) = request.headers.get("x-real-ip") {
        let trimmed = real_ip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(peer) = request.headers.get("x-robominer-peer") {
        let trimmed = peer.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "unknown".to_string()
}

pub(crate) fn normalize_login_key(login_name: &str) -> String {
    login_name.trim().to_ascii_lowercase()
}

/// Returns true when the client should receive HTTP 429 before auth work runs.
pub(crate) fn auth_attempt_is_rate_limited(ip: &str, login_name: &str) -> bool {
    let login_key = normalize_login_key(login_name);
    let now = Instant::now();
    auth_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .is_limited(ip, &login_key, now)
}

pub(crate) fn record_auth_attempt(ip: &str, login_name: &str) {
    let login_key = normalize_login_key(login_name);
    let now = Instant::now();
    auth_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record(ip, &login_key, now);
}

pub(crate) fn log_auth_failure(ip: &str, login_name: &str, result: &str) {
    let safe_login = sanitize_log_token(login_name);
    eprintln!("auth_failure ip={ip} login_name={safe_login} result={result}");
}

fn sanitize_log_token(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    trimmed
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '@' | '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .take(64)
        .collect()
}

#[cfg(test)]
pub(crate) fn reset_auth_rate_limiter_for_tests() {
    let mut limiter = auth_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    limiter.by_ip.clear();
    limiter.by_login.clear();
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn client_ip_prefers_forwarded_for() {
        let mut request = Request {
            method: "POST".to_string(),
            path: "/login".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::from([
                (
                    "x-forwarded-for".to_string(),
                    "203.0.113.9, 10.0.0.1".to_string(),
                ),
                ("x-real-ip".to_string(), "10.0.0.2".to_string()),
                ("x-robominer-peer".to_string(), "127.0.0.1".to_string()),
            ]),
        };
        assert_eq!(client_ip(&request), "203.0.113.9");
        request.headers.remove("x-forwarded-for");
        assert_eq!(client_ip(&request), "10.0.0.2");
        request.headers.remove("x-real-ip");
        assert_eq!(client_ip(&request), "127.0.0.1");
        request.headers.clear();
        assert_eq!(client_ip(&request), "unknown");
    }

    #[test]
    fn auth_rate_limiter_trips_on_ip_and_login_windows() {
        reset_auth_rate_limiter_for_tests();
        let ip = "198.51.100.20";
        for index in 0..MAX_ATTEMPTS_PER_IP {
            // Distinct login keys so the IP window trips first.
            let login = format!("alice-{index}");
            assert!(!auth_attempt_is_rate_limited(ip, &login));
            record_auth_attempt(ip, &login);
        }
        assert!(auth_attempt_is_rate_limited(ip, "alice-next"));

        reset_auth_rate_limiter_for_tests();
        for index in 0..MAX_ATTEMPTS_PER_LOGIN {
            let ip = format!("198.51.100.{index}");
            assert!(!auth_attempt_is_rate_limited(&ip, "bob"));
            record_auth_attempt(&ip, "bob");
        }
        assert!(auth_attempt_is_rate_limited("203.0.113.1", "bob"));
    }
}
