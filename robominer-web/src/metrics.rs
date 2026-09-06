//! Lightweight process metrics for loopback scrapes (`GET /metrics`).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::http::{Request, Response};

static HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static AUTH_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);
static SIGNUP_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);

pub fn record_http_request() {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_auth_failure() {
    AUTH_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_signup_failure() {
    SIGNUP_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Prometheus text exposition. Intended for loopback-only scrapes.
pub fn metrics_response() -> Response {
    let body = format!(
        "# HELP robominer_http_requests_total HTTP requests handled by robominer-web.\n\
         # TYPE robominer_http_requests_total counter\n\
         robominer_http_requests_total {}\n\
         # HELP robominer_auth_failures_total Failed login attempts.\n\
         # TYPE robominer_auth_failures_total counter\n\
         robominer_auth_failures_total {}\n\
         # HELP robominer_signup_failures_total Failed signup attempts.\n\
         # TYPE robominer_signup_failures_total counter\n\
         robominer_signup_failures_total {}\n",
        HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed),
        AUTH_FAILURES_TOTAL.load(Ordering::Relaxed),
        SIGNUP_FAILURES_TOTAL.load(Ordering::Relaxed),
    );
    Response {
        status: 200,
        reason: "OK",
        content_type: "text/plain; version=0.0.4; charset=utf-8",
        headers: vec![("Cache-Control", "no-store".to_string())],
        body: Arc::from(body.into_bytes()),
    }
}

pub fn is_loopback_peer(request: &Request) -> bool {
    request
        .headers
        .get("x-robominer-peer")
        .map(|peer| peer == "127.0.0.1" || peer == "::1" || peer.starts_with("127."))
        .unwrap_or(false)
}
