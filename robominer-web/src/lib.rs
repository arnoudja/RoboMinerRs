#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! HTTP host for RoboMiner: Axum transport shell, session/CSRF, HTML pages, and
//! static assets. Game mutations and rejection copy live in `robominer-domain`;
//! SQL and typed contracts live in `robominer-db`. See `CONTRIBUTING.md`.

use std::future::Future;
use std::path::PathBuf;
use std::sync::OnceLock;

mod account_page;
mod achievements_page;
mod animation_script;
mod app_shell;
mod auth_pages;
mod csrf;
mod edit_code_page;
mod health;
mod help_pages;
mod html;
mod http;
mod leaderboard_page;
mod metrics;
mod mining_area_atlas;
mod mining_area_overview_page;
mod mining_queue_page;
mod mining_results_page;
mod page_context;
mod percent_encode;
mod rally_pages;
mod rate_limit;
mod request_helpers;
mod robot_page;
mod robot_stats_page;
mod router;
mod routes;
mod server;
mod session;
mod settings;
mod shop_page;
pub mod startup;
mod static_assets;
mod static_files;

pub use server::serve;
pub use settings::{WebSettings, web_settings};
pub fn configure_session_secret(secret: &str) -> Result<(), String> {
    session::configure_session_secret(secret)
}
pub fn configure_secure_cookies(enabled: bool) {
    session::configure_secure_cookies(enabled);
}
pub fn configure_session_ttl_secs(ttl_secs: u64) {
    session::configure_session_ttl_secs(ttl_secs);
}
pub fn resolve_session_ttl_secs(
    env_secs: Option<&str>,
    env_hours: Option<&str>,
) -> Result<u64, String> {
    session::resolve_session_ttl_secs(env_secs, env_hours)
}
pub fn resolve_session_secret(
    configured: Option<&str>,
    bind_host: &str,
    allow_insecure_dev_secret: bool,
) -> Result<String, &'static str> {
    session::resolve_session_secret(configured, bind_host, allow_insecure_dev_secret)
}
pub use http::{Request, Response};
pub(crate) use request_helpers::{
    is_post, mutation_form_has, mutation_i64, query_i64, query_signed_i64, request_user_id,
    session_username,
};
pub use router::route;
pub use session::{resolve_secure_cookies, validate_trust_proxy_bind};

static DATABASE_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub static_root: PathBuf,
    pub database_pool: Option<robominer_db::MySqlPool>,
    /// When false, the sign-up form is hidden and create-user POSTs are rejected.
    /// Config/env default is off (`allowsignup` / `ROBOMINER_ALLOW_SIGNUP` unset).
    pub allow_signup: bool,
    /// When true, trust only `X-Real-Ip` for client IP (rate limits, auth logs).
    /// Enable only behind a reverse proxy that sets that header to `$remote_addr`.
    /// Requires loopback bind and Secure cookies.
    pub trust_proxy: bool,
}

/// Sync bridge for process startup (e.g. DB pool connect in `main`).
/// Request handlers should `await` futures on the Axum Tokio runtime instead.
pub fn block_on_database<F>(future: F) -> F::Output
where
    F: Future,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }

    DATABASE_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap_or_else(|error| panic!("database runtime should initialize: {error}"))
        })
        .block_on(future)
}

#[cfg(test)]
mod block_on_database_tests {
    use super::block_on_database;

    #[test]
    fn block_on_database_runs_without_current_runtime() {
        let value = block_on_database(async { 41_i32 + 1 });
        assert_eq!(value, 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn block_on_database_runs_inside_tokio_runtime() {
        let value = block_on_database(async { 20_i32 + 2 });
        assert_eq!(value, 22);
    }
}

/// Integration-test helpers for routing against a real database pool.
/// Available in debug builds and `cargo test` only — not in release binaries.
#[cfg(any(test, debug_assertions))]
#[doc(hidden)]
pub mod test_support {
    use std::collections::HashMap;

    pub use crate::csrf::csrf_token_from_cookie;
    pub use crate::{Request, Response, ServerConfig, configure_session_secret, route};

    pub fn format_authenticated_cookie(user_id: i64, username: &str) -> String {
        crate::session::format_authenticated_cookie(user_id, username)
    }

    pub fn user_id_from_cookie_header(cookies: &str) -> Option<i64> {
        let request = Request {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::from([("cookie".to_string(), cookies.to_string())]),
        };
        crate::request_user_id(&request)
    }

    pub const MAX_MUTATIONS_PER_USER_ACTION: usize =
        crate::rate_limit::MAX_MUTATIONS_PER_USER_ACTION;

    pub fn reset_mutation_rate_limiter_for_tests() {
        crate::rate_limit::reset_mutation_rate_limiter_for_tests();
    }

    pub const MAX_ATTEMPTS_PER_LOGIN: usize = crate::rate_limit::MAX_ATTEMPTS_PER_LOGIN;

    pub fn reset_auth_rate_limiter_for_tests() {
        crate::rate_limit::reset_auth_rate_limiter_for_tests();
    }

    pub fn lock_auth_rate_limiter_for_tests() -> std::sync::MutexGuard<'static, ()> {
        crate::rate_limit::lock_auth_rate_limiter_for_tests()
    }

    pub fn record_auth_attempt(ip: &str, login_name: &str) {
        crate::rate_limit::record_auth_attempt(ip, login_name);
    }
}
