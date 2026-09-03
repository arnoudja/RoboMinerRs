//! Auth and mutation rate limiting for login and authenticated POST handlers.

mod auth;
mod mutation;

#[cfg(any(test, debug_assertions))]
pub(crate) use auth::lock_auth_rate_limiter_for_tests;
#[cfg(any(test, debug_assertions))]
pub(crate) use auth::reset_auth_rate_limiter_for_tests;
#[allow(unused_imports)]
pub(crate) use auth::{
    MAX_ATTEMPTS_PER_IP, MAX_ATTEMPTS_PER_LOGIN, auth_attempt_is_rate_limited, log_auth_failure,
    normalize_login_key, record_auth_attempt,
};
#[cfg(any(test, debug_assertions))]
pub(crate) use mutation::MAX_MUTATIONS_PER_USER_ACTION;
pub(crate) use mutation::reject_rate_limited_mutation;
#[cfg(any(test, debug_assertions))]
pub(crate) use mutation::reset_mutation_rate_limiter_for_tests;

use crate::http::Request;

/// Client IP for rate limiting / auth logs.
///
/// When `trust_proxy` is true (behind a reverse proxy that sets client headers),
/// prefers `X-Real-Ip` (proxy-set `$remote_addr`) then the first
/// `X-Forwarded-For` hop. Preferring Real-IP avoids client-spoofed XFF when the
/// proxy also sets Real-IP (nginx/Caddy examples overwrite both). Otherwise uses
/// the peer address injected by the Axum acceptor (`x-robominer-peer`).
pub(crate) fn client_ip(request: &Request, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(real_ip) = request.headers.get("x-real-ip") {
            let trimmed = real_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(forwarded) = request.headers.get("x-forwarded-for")
            && let Some(first) = forwarded.split(',').next()
        {
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
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

pub(super) fn sanitize_log_token(value: &str) -> String {
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
mod tests {
    use std::collections::{HashMap, VecDeque};
    use std::time::{Duration, Instant};

    use super::auth::{
        AUTH_WINDOW, MAX_ATTEMPTS_PER_IP, MAX_ATTEMPTS_PER_LOGIN, auth_rate_limiter,
        lock_auth_rate_limiter_for_tests,
    };
    use super::mutation::{
        lock_mutation_rate_limiter_for_tests, mutation_action_family,
        mutation_attempt_is_rate_limited, record_mutation_attempt,
    };
    use super::*;

    fn request_with_headers(headers: HashMap<String, String>) -> Request {
        Request {
            method: "POST".to_string(),
            path: "/login".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers,
        }
    }

    #[test]
    fn client_ip_ignores_proxy_headers_unless_trusted() {
        let request = request_with_headers(HashMap::from([
            (
                "x-forwarded-for".to_string(),
                "203.0.113.9, 10.0.0.1".to_string(),
            ),
            ("x-real-ip".to_string(), "10.0.0.2".to_string()),
            ("x-robominer-peer".to_string(), "127.0.0.1".to_string()),
        ]));
        assert_eq!(client_ip(&request, false), "127.0.0.1");
        // Prefer X-Real-IP (set to $remote_addr by the proxy) over X-Forwarded-For.
        assert_eq!(client_ip(&request, true), "10.0.0.2");
    }

    #[test]
    fn client_ip_prefers_real_ip_over_spoofed_forwarded_for() {
        // Client-supplied XFF must not win when the proxy also set X-Real-IP.
        let request = request_with_headers(HashMap::from([
            (
                "x-forwarded-for".to_string(),
                "198.51.100.99, 203.0.113.10".to_string(),
            ),
            ("x-real-ip".to_string(), "203.0.113.10".to_string()),
            ("x-robominer-peer".to_string(), "127.0.0.1".to_string()),
        ]));
        assert_eq!(client_ip(&request, true), "203.0.113.10");
    }

    #[test]
    fn client_ip_falls_back_through_trusted_proxy_headers() {
        let mut request = request_with_headers(HashMap::from([
            (
                "x-forwarded-for".to_string(),
                "203.0.113.9, 10.0.0.1".to_string(),
            ),
            ("x-real-ip".to_string(), "10.0.0.2".to_string()),
            ("x-robominer-peer".to_string(), "127.0.0.1".to_string()),
        ]));
        assert_eq!(client_ip(&request, true), "10.0.0.2");
        request.headers.remove("x-real-ip");
        assert_eq!(client_ip(&request, true), "203.0.113.9");
        request.headers.remove("x-forwarded-for");
        assert_eq!(client_ip(&request, true), "127.0.0.1");
        request.headers.clear();
        assert_eq!(client_ip(&request, true), "unknown");
    }

    #[test]
    fn auth_rate_limiter_trips_on_ip_and_login_windows() {
        let _guard = lock_auth_rate_limiter_for_tests();
        reset_auth_rate_limiter_for_tests();
        let ip = "198.51.100.20";
        for index in 0..MAX_ATTEMPTS_PER_IP {
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

    #[test]
    fn auth_rate_limiter_drops_empty_keys_after_prune() {
        let _guard = lock_auth_rate_limiter_for_tests();
        reset_auth_rate_limiter_for_tests();
        let now = Instant::now();
        let expired = now - AUTH_WINDOW - Duration::from_secs(1);
        {
            let mut limiter = auth_rate_limiter()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            limiter
                .by_ip
                .insert("198.51.100.50".to_string(), VecDeque::from([expired]));
            limiter
                .by_login
                .insert("stale".to_string(), VecDeque::from([expired]));
            limiter.last_sweep = None;
            limiter.sweep_expired(now);
            assert!(limiter.by_ip.is_empty());
            assert!(limiter.by_login.is_empty());
        }
    }

    #[test]
    fn mutation_rate_limiter_is_per_user_and_action_family() {
        let _guard = lock_mutation_rate_limiter_for_tests();
        reset_mutation_rate_limiter_for_tests();
        let user_id = 99_i64;
        for _ in 0..MAX_MUTATIONS_PER_USER_ACTION {
            assert!(!mutation_attempt_is_rate_limited(user_id, "shop"));
            record_mutation_attempt(user_id, "shop");
        }
        assert!(mutation_attempt_is_rate_limited(user_id, "shop"));
        assert!(!mutation_attempt_is_rate_limited(user_id, "robot"));
        assert!(!mutation_attempt_is_rate_limited(user_id + 1, "shop"));
    }

    #[test]
    fn mutation_action_family_maps_known_paths() {
        assert_eq!(mutation_action_family("/shop"), "shop");
        assert_eq!(mutation_action_family("/MiningQueue"), "mining_queue");
        assert_eq!(mutation_action_family("/editCode"), "edit_code");
        assert_eq!(mutation_action_family("/unknown"), "other");
    }
}
