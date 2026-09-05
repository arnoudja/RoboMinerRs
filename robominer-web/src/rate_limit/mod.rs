//! Auth and mutation rate limiting for login and authenticated POST handlers.

mod auth;
mod mutation;

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Soft ceiling on distinct keys retained per in-process rate-limit map.
///
/// Process-local limiters are best-effort (no shared store). Under scanning,
/// unique IPs/logins can accumulate until windows expire; this drops the
/// coldest keys (oldest front timestamp) once a map grows past the cap.
pub(super) const MAX_TRACKED_KEYS: usize = 4096;

pub(super) fn enforce_map_cap<K: Eq + std::hash::Hash + Clone>(
    map: &mut HashMap<K, VecDeque<Instant>>,
    max_keys: usize,
) {
    if map.len() <= max_keys {
        return;
    }
    let overflow = map.len() - max_keys;
    let mut coldest: Vec<(K, Instant)> = map
        .iter()
        .filter_map(|(key, window)| window.front().map(|instant| (key.clone(), *instant)))
        .collect();
    coldest.sort_by_key(|(_, instant)| *instant);
    for (key, _) in coldest.into_iter().take(overflow) {
        map.remove(&key);
    }
}

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

/// Rate-limit / auth-log key when `trust_proxy` is on but `X-Real-Ip` is absent
/// or blank. Avoids collapsing every client onto the loopback peer.
pub(crate) const PROXY_MISSING_REAL_IP_KEY: &str = "proxy-missing-real-ip";

/// Client IP for rate limiting / auth logs.
///
/// When `trust_proxy` is true (behind a reverse proxy that sets client headers),
/// uses only `X-Real-Ip` (proxy-set `$remote_addr`). Falling back to
/// `X-Forwarded-For` is intentionally omitted: misconfigured proxies that
/// append client-supplied XFF would reintroduce spoofable rate-limit keys.
/// Missing/blank Real-IP uses [`PROXY_MISSING_REAL_IP_KEY`] (and logs an error)
/// instead of the peer address, so a misconfigured proxy does not share one
/// bucket across all clients. Otherwise uses the peer address injected by the
/// Axum acceptor (`x-robominer-peer`).
pub(crate) fn client_ip(request: &Request, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(real_ip) = request.headers.get("x-real-ip") {
            let trimmed = real_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        tracing::error!(
            path = %request.path,
            "trustproxy enabled but X-Real-Ip missing or blank; using dedicated rate-limit key"
        );
        return PROXY_MISSING_REAL_IP_KEY.to_string();
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
        MAX_MUTATIONS_PER_IP, lock_mutation_rate_limiter_for_tests, mutation_action_family,
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
    fn client_ip_with_trust_proxy_ignores_xff_and_uses_peer_without_real_ip() {
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
        // Spoofable XFF must not become the rate-limit key; missing Real-IP must
        // not collapse onto the loopback peer either.
        assert_eq!(client_ip(&request, true), PROXY_MISSING_REAL_IP_KEY);
        request.headers.remove("x-forwarded-for");
        assert_eq!(client_ip(&request, true), PROXY_MISSING_REAL_IP_KEY);
        request
            .headers
            .insert("x-real-ip".to_string(), "   ".to_string());
        assert_eq!(client_ip(&request, true), PROXY_MISSING_REAL_IP_KEY);
        request.headers.clear();
        assert_eq!(client_ip(&request, true), PROXY_MISSING_REAL_IP_KEY);
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
        let ip = "198.51.100.40";
        for _ in 0..MAX_MUTATIONS_PER_USER_ACTION {
            assert!(!mutation_attempt_is_rate_limited(ip, user_id, "shop"));
            record_mutation_attempt(ip, user_id, "shop");
        }
        assert!(mutation_attempt_is_rate_limited(ip, user_id, "shop"));
        assert!(!mutation_attempt_is_rate_limited(ip, user_id, "robot"));
        assert!(!mutation_attempt_is_rate_limited(ip, user_id + 1, "shop"));
    }

    #[test]
    fn mutation_rate_limiter_trips_on_shared_ip_budget() {
        let _guard = lock_mutation_rate_limiter_for_tests();
        reset_mutation_rate_limiter_for_tests();
        let ip = "198.51.100.41";
        for index in 0..MAX_MUTATIONS_PER_IP {
            let user_id = i64::from(index as u32);
            // Spread across action families so the user/action bucket is not the tripwire.
            let action = if index % 2 == 0 { "shop" } else { "robot" };
            assert!(!mutation_attempt_is_rate_limited(ip, user_id, action));
            record_mutation_attempt(ip, user_id, action);
        }
        assert!(mutation_attempt_is_rate_limited(ip, 9_999, "edit_code"));
        assert!(!mutation_attempt_is_rate_limited(
            "203.0.113.9",
            9_999,
            "edit_code"
        ));
    }

    #[test]
    fn mutation_action_family_maps_known_paths() {
        assert_eq!(mutation_action_family("/shop"), "shop");
        assert_eq!(mutation_action_family("/miningQueue"), "mining_queue");
        assert_eq!(mutation_action_family("/editCode"), "edit_code");
        assert_eq!(mutation_action_family("/unknown"), "other");
    }

    #[test]
    fn enforce_map_cap_evicts_coldest_keys() {
        use std::collections::{HashMap, VecDeque};
        use std::time::{Duration, Instant};

        let base = Instant::now();
        let mut map: HashMap<String, VecDeque<Instant>> = HashMap::new();
        for index in 0..10 {
            let mut window = VecDeque::new();
            window.push_back(base + Duration::from_secs(index));
            map.insert(format!("ip-{index}"), window);
        }
        enforce_map_cap(&mut map, 4);
        assert_eq!(map.len(), 4);
        // Coldest keys (0..5) should be gone; newest remain.
        for index in 0..6 {
            assert!(
                !map.contains_key(&format!("ip-{index}")),
                "expected cold key ip-{index} evicted"
            );
        }
        for index in 6..10 {
            assert!(
                map.contains_key(&format!("ip-{index}")),
                "expected warm key ip-{index} retained"
            );
        }
    }
}
