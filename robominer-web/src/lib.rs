use std::future::Future;
use std::path::PathBuf;
use std::sync::OnceLock;

mod account_page;
mod achievements_page;
mod animation_script;
mod app_shell;
mod auth_pages;
mod edit_code_page;
mod help_page;
mod help_pages;
mod html;
mod http;
mod leaderboard_page;
mod mining_area_atlas;
mod mining_area_overview_page;
mod mining_queue_page;
mod mining_results_page;
mod rally_pages;
mod request_helpers;
mod robot_page;
mod router;
mod session;
mod shop_page;

pub use http::serve;
pub fn configure_session_secret(secret: &str) {
    session::configure_session_secret(secret);
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
    config_secs: Option<&str>,
    config_hours: Option<&str>,
) -> Result<u64, String> {
    session::resolve_session_ttl_secs(env_secs, env_hours, config_secs, config_hours)
}
pub fn resolve_session_secret(
    configured: Option<&str>,
    bind_host: &str,
) -> Result<String, &'static str> {
    session::resolve_session_secret(configured, bind_host)
}
pub use http::{Request, Response};
pub(crate) use request_helpers::{
    login_redirect, query_i64, query_signed_i64, request_user_id, session_username,
};
pub use router::route;

static DATABASE_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub static_root: PathBuf,
    pub database_pool: Option<robominer_db::MySqlPool>,
    /// When false, the sign-up form is hidden and create-user POSTs are rejected.
    pub allow_signup: bool,
}

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
                .expect("database runtime should initialize")
        })
        .block_on(future)
}

/// Integration-test helpers for routing against a real database pool.
#[doc(hidden)]
pub mod test_support {
    pub use crate::{Request, Response, ServerConfig, configure_session_secret, route};
}
