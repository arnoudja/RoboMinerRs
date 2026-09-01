use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub(crate) const AUTH_WINDOW: Duration = Duration::from_secs(60);
pub(crate) const MAX_ATTEMPTS_PER_IP: usize = 10;
pub(crate) const MAX_ATTEMPTS_PER_LOGIN: usize = 5;

static AUTH_RATE_LIMITER: OnceLock<Mutex<AuthRateLimiter>> = OnceLock::new();

#[derive(Debug, Default)]
pub(super) struct AuthRateLimiter {
    pub(super) by_ip: HashMap<String, VecDeque<Instant>>,
    pub(super) by_login: HashMap<String, VecDeque<Instant>>,
    pub(super) last_sweep: Option<Instant>,
}

pub(super) fn auth_rate_limiter() -> &'static Mutex<AuthRateLimiter> {
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

    pub(super) fn sweep_expired(&mut self, now: Instant) {
        let should_sweep = self
            .last_sweep
            .is_none_or(|last| now.duration_since(last) >= AUTH_WINDOW);
        if !should_sweep {
            return;
        }
        self.by_ip.retain(|_, window| {
            Self::prune(window, now);
            !window.is_empty()
        });
        self.by_login.retain(|_, window| {
            Self::prune(window, now);
            !window.is_empty()
        });
        self.last_sweep = Some(now);
    }

    fn window_len(map: &mut HashMap<String, VecDeque<Instant>>, key: &str, now: Instant) -> usize {
        let Some(window) = map.get_mut(key) else {
            return 0;
        };
        Self::prune(window, now);
        let len = window.len();
        if len == 0 {
            map.remove(key);
            return 0;
        }
        len
    }

    fn is_limited(&mut self, ip: &str, login_key: &str, now: Instant) -> bool {
        self.sweep_expired(now);
        if Self::window_len(&mut self.by_ip, ip, now) >= MAX_ATTEMPTS_PER_IP {
            return true;
        }

        if login_key.is_empty() {
            return false;
        }
        Self::window_len(&mut self.by_login, login_key, now) >= MAX_ATTEMPTS_PER_LOGIN
    }

    fn record(&mut self, ip: &str, login_key: &str, now: Instant) {
        self.sweep_expired(now);
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
    let safe_login = super::sanitize_log_token(login_name);
    tracing::warn!(
        ip = %ip,
        login_name = %safe_login,
        result = %result,
        "auth_failure"
    );
}

#[cfg(any(test, debug_assertions))]
pub(crate) fn reset_auth_rate_limiter_for_tests() {
    let mut limiter = auth_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    limiter.by_ip.clear();
    limiter.by_login.clear();
    limiter.last_sweep = None;
}

#[cfg(any(test, debug_assertions))]
pub(crate) fn lock_auth_rate_limiter_for_tests() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
