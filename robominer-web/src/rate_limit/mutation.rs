use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::http::{Request, Response};
use crate::routes::AppRoute;

const MUTATION_WINDOW: Duration = Duration::from_secs(60);
pub(crate) const MAX_MUTATIONS_PER_USER_ACTION: usize = 30;

static MUTATION_RATE_LIMITER: OnceLock<Mutex<MutationRateLimiter>> = OnceLock::new();

#[derive(Debug, Default)]
struct MutationRateLimiter {
    by_user_action: HashMap<(i64, &'static str), VecDeque<Instant>>,
    last_sweep: Option<Instant>,
}

fn mutation_rate_limiter() -> &'static Mutex<MutationRateLimiter> {
    MUTATION_RATE_LIMITER.get_or_init(|| Mutex::new(MutationRateLimiter::default()))
}

impl MutationRateLimiter {
    fn prune(window: &mut VecDeque<Instant>, now: Instant) {
        while window
            .front()
            .is_some_and(|instant| now.duration_since(*instant) > MUTATION_WINDOW)
        {
            window.pop_front();
        }
    }

    fn sweep_expired(&mut self, now: Instant) {
        let should_sweep = self
            .last_sweep
            .is_none_or(|last| now.duration_since(last) >= MUTATION_WINDOW);
        if !should_sweep {
            return;
        }
        self.by_user_action.retain(|_, window| {
            Self::prune(window, now);
            !window.is_empty()
        });
        self.last_sweep = Some(now);
    }

    fn is_limited(&mut self, user_id: i64, action: &'static str, now: Instant) -> bool {
        self.sweep_expired(now);
        let Some(window) = self.by_user_action.get_mut(&(user_id, action)) else {
            return false;
        };
        Self::prune(window, now);
        let len = window.len();
        if len == 0 {
            self.by_user_action.remove(&(user_id, action));
            return false;
        }
        len >= MAX_MUTATIONS_PER_USER_ACTION
    }

    fn record(&mut self, user_id: i64, action: &'static str, now: Instant) {
        self.sweep_expired(now);
        let window = self.by_user_action.entry((user_id, action)).or_default();
        Self::prune(window, now);
        window.push_back(now);
    }
}

/// Action family for authenticated mutation rate limiting (keyed with user id).
pub(crate) fn mutation_action_family(path: &str) -> &'static str {
    match AppRoute::from_path(path) {
        Some(AppRoute::Shop) => "shop",
        Some(AppRoute::MiningQueue) => "mining_queue",
        Some(AppRoute::EditCode) => "edit_code",
        Some(AppRoute::Achievements) => "achievements",
        Some(AppRoute::Robot) => "robot",
        Some(AppRoute::Account) => "account",
        _ => "other",
    }
}

/// Returns true when an authenticated mutation should receive HTTP 429.
pub(crate) fn mutation_attempt_is_rate_limited(user_id: i64, action: &'static str) -> bool {
    let now = Instant::now();
    mutation_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .is_limited(user_id, action, now)
}

pub(crate) fn record_mutation_attempt(user_id: i64, action: &'static str) {
    let now = Instant::now();
    mutation_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record(user_id, action, now);
}

pub(crate) fn reject_rate_limited_mutation(request: &Request, user_id: i64) -> Option<Response> {
    let action = mutation_action_family(&request.path);
    if mutation_attempt_is_rate_limited(user_id, action) {
        return Some(Response::too_many_requests(
            "Too many requests. Please wait a moment and try again.",
        ));
    }
    record_mutation_attempt(user_id, action);
    None
}

#[cfg(any(test, debug_assertions))]
pub(crate) fn reset_mutation_rate_limiter_for_tests() {
    let mut limiter = mutation_rate_limiter()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    limiter.by_user_action.clear();
    limiter.last_sweep = None;
}

#[cfg(test)]
pub(crate) fn lock_mutation_rate_limiter_for_tests() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
