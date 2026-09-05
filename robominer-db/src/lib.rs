#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Persistence layer: SQL, migrations, pool/config, record DTOs, typed
//! `*Request` / `*Rejection` contracts, and **transactional multi-table game
//! rules** (shop economics, queue capacity, achievement claims, claim tax).
//! Loadouts, simulation/verify façades, and rejection copy live in
//! `robominer-domain`. See `CONTRIBUTING.md` and `docs/architecture.md`.
//!
//! Public surface stays flat for compatibility (`pub use module::*` →
//! `robominer_db::enqueue_mining`). **No new root wildcards** — do not add
//! another `pub use some_module::*` at this crate root. Prefer
//! `robominer_db::module::…` for new APIs and when updating hot call sites;
//! export new symbols from the owning module only (existing root re-exports
//! stay).

pub use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

mod initial_ore_wallet_max;
mod query_util;

pub use initial_ore_wallet_max::INITIAL_ORE_WALLET_MAX;
pub use query_util::{assert_sql_safe, in_placeholders};

pub const SCORE_HISTORY_FACTOR: f64 = 5.0;
pub const SCORE_START_FACTOR: f64 = 1.4;
pub const DEFAULT_MAX_CONNECTIONS: u32 = 5;

pub mod achievements;
pub mod activity;
pub mod app_shell;
pub mod assets;
pub mod catalog;
pub mod config;
pub mod leaderboard;
pub mod migrate;
pub mod mining_areas;
pub mod mining_queue;
mod password;
pub mod pool;
pub mod program_sources;
pub mod rally;
pub mod results;
pub mod robots;
pub mod shop;
mod types;
pub mod users;

pub use achievements::*;
pub use activity::*;
pub use app_shell::*;
pub use assets::*;
pub use catalog::*;
pub use config::*;
pub use leaderboard::*;
pub use migrate::*;
pub use mining_areas::*;
pub use mining_queue::*;
pub use pool::*;
pub use program_sources::*;
pub use rally::*;
pub use results::*;
pub use robots::*;
pub use shop::*;
pub use types::*;
pub use users::*;

pub async fn connect(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    connect_with_max_connections(database_url, DEFAULT_MAX_CONNECTIONS).await
}

/// Cheap connectivity probe used by `/health` and similar readiness checks.
pub async fn ping(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}

pub async fn connect_with_max_connections(
    database_url: &str,
    max_connections: u32,
) -> Result<MySqlPool, sqlx::Error> {
    ensure_remote_mysql_tls(database_url)?;
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
}

/// Refuse non-loopback MySQL URLs that do not request TLS, unless
/// `ROBOMINER_ALLOW_INSECURE_MYSQL=1` is set.
fn ensure_remote_mysql_tls(database_url: &str) -> Result<(), sqlx::Error> {
    ensure_remote_mysql_tls_with_allow(database_url, insecure_mysql_allowed())
}

fn insecure_mysql_allowed() -> bool {
    matches!(
        std::env::var("ROBOMINER_ALLOW_INSECURE_MYSQL").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

fn ensure_remote_mysql_tls_with_allow(
    database_url: &str,
    allow_insecure: bool,
) -> Result<(), sqlx::Error> {
    let Some(host) = mysql_url_host(database_url) else {
        return Ok(());
    };
    if matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") {
        return Ok(());
    }
    if mysql_url_requests_tls(database_url) {
        return Ok(());
    }
    if allow_insecure {
        tracing::warn!(
            host = %host,
            "MySQL host is not loopback and URL does not request TLS; \
             continuing because ROBOMINER_ALLOW_INSECURE_MYSQL is set"
        );
        return Ok(());
    }
    Err(sqlx::Error::Configuration(
        format!(
            "MySQL host {host:?} is not loopback and URL does not request TLS \
             (add ?ssl-mode=REQUIRED, or set ROBOMINER_ALLOW_INSECURE_MYSQL=1)"
        )
        .into(),
    ))
}

fn mysql_url_requests_tls(database_url: &str) -> bool {
    let query = database_url
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or("");
    query.split('&').any(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = key.to_ascii_lowercase();
        let value = value.to_ascii_lowercase();
        matches!(
            (key.as_str(), value.as_str()),
            ("ssl-mode", "required" | "verify_ca" | "verify_identity")
                | ("sslmode", "require" | "verify-ca" | "verify-full")
        )
    })
}

fn mysql_url_host(database_url: &str) -> Option<String> {
    let rest = database_url.strip_prefix("mysql://")?;
    let authority = rest.split_once('/').map(|(a, _)| a).unwrap_or(rest);
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    if let Some(host) = host_port.strip_prefix('[') {
        return host.split_once(']').map(|(h, _)| h.to_string());
    }
    Some(
        host_port
            .rsplit_once(':')
            .map(|(h, _)| h)
            .unwrap_or(host_port)
            .to_string(),
    )
}

/// Shared stderr `EnvFilter` tracing init for binaries (web + engine).
pub fn init_default_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}

/// Resolve pool size from env (`ROBOMINER_DB_MAX_CONNECTIONS`).
pub fn resolve_max_connections(env_value: Option<&str>) -> Result<u32, String> {
    let raw = env_value.map(str::trim).filter(|value| !value.is_empty());
    let Some(raw) = raw else {
        return Ok(DEFAULT_MAX_CONNECTIONS);
    };

    let max_connections = raw
        .parse::<u32>()
        .map_err(|_| format!("db max connections must be a positive integer, got {raw:?}"))?;
    if max_connections == 0 {
        return Err("db max connections must be greater than 0".to_string());
    }
    Ok(max_connections)
}

#[cfg(test)]
mod tests {
    use crate::mining_areas::{MiningRallyQueueRow, mining_rally_queue_rows};
    use crate::pool::{PoolItemRow, next_pool_rally_item_rows};
    use crate::{
        DEFAULT_MAX_CONNECTIONS, ensure_remote_mysql_tls_with_allow, mysql_url_host,
        mysql_url_requests_tls, resolve_max_connections,
    };

    #[test]
    fn resolve_max_connections_defaults_and_reads_env() {
        assert_eq!(
            resolve_max_connections(None).expect("default"),
            DEFAULT_MAX_CONNECTIONS
        );
        assert_eq!(resolve_max_connections(Some("20")).expect("env"), 20);
        assert_eq!(
            resolve_max_connections(Some("")).expect("empty env uses default"),
            DEFAULT_MAX_CONNECTIONS
        );
    }

    #[test]
    fn resolve_max_connections_rejects_invalid_values() {
        assert!(resolve_max_connections(Some("0")).is_err());
        assert!(resolve_max_connections(Some("abc")).is_err());
    }

    #[test]
    fn mysql_url_host_parses_auth_and_ipv6() {
        assert_eq!(
            mysql_url_host("mysql://u:p@db.example.com:3306/RoboMiner").as_deref(),
            Some("db.example.com")
        );
        assert_eq!(
            mysql_url_host("mysql://robominer:password@127.0.0.1:3306/RoboMiner").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(
            mysql_url_host("mysql://u:p@[::1]:3306/RoboMiner").as_deref(),
            Some("::1")
        );
    }

    #[test]
    fn mysql_url_requests_tls_detects_ssl_mode_query() {
        assert!(mysql_url_requests_tls(
            "mysql://u:p@db.example.com/RoboMiner?ssl-mode=REQUIRED"
        ));
        assert!(mysql_url_requests_tls(
            "mysql://u:p@db.example.com/RoboMiner?sslmode=verify-full"
        ));
        assert!(!mysql_url_requests_tls(
            "mysql://u:p@db.example.com/RoboMiner"
        ));
        assert!(!mysql_url_requests_tls(
            "mysql://u:p@db.example.com/RoboMiner?ssl-mode=DISABLED"
        ));
    }

    #[test]
    fn remote_mysql_without_tls_is_rejected_unless_allowlisted() {
        let remote = "mysql://u:p@db.example.com:3306/RoboMiner";
        assert!(ensure_remote_mysql_tls_with_allow(remote, false).is_err());
        assert!(ensure_remote_mysql_tls_with_allow(remote, true).is_ok());
        assert!(
            ensure_remote_mysql_tls_with_allow(
                "mysql://u:p@db.example.com/RoboMiner?ssl-mode=REQUIRED",
                false
            )
            .is_ok()
        );
        assert!(
            ensure_remote_mysql_tls_with_allow(
                "mysql://robominer:password@127.0.0.1:3306/RoboMiner",
                false
            )
            .is_ok()
        );
        assert!(
            ensure_remote_mysql_tls_with_allow(
                "mysql://robominer:password@localhost/RoboMiner",
                false
            )
            .is_ok()
        );
    }

    #[test]
    fn next_pool_rally_items_keep_only_lowest_runs_done_cohort() {
        let rows = vec![
            pool_item_row(1, 900, 11, 50.0, 2),
            pool_item_row(2, 900, 12, 80.0, 2),
            pool_item_row(3, 900, 13, 120.0, 3),
        ];

        let items = next_pool_rally_item_rows(rows);

        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn next_pool_rally_items_allow_empty_pools() {
        assert!(next_pool_rally_item_rows(Vec::new()).is_empty());
    }

    #[test]
    fn next_mining_rally_queue_keeps_first_robot_per_user_before_cap() {
        let rows = vec![
            mining_rally_queue_row(1, 100, 11, 501, 5),
            mining_rally_queue_row(2, 100, 12, 502, 6),
            mining_rally_queue_row(3, 100, 13, 501, 7),
            mining_rally_queue_row(4, 100, 14, 503, 8),
            mining_rally_queue_row(5, 100, 15, 504, 9),
            mining_rally_queue_row(6, 100, 16, 505, 10),
        ];

        let queue = mining_rally_queue_rows(rows);

        assert_eq!(
            queue
                .iter()
                .map(|record| (record.queue.id, record.user_id))
                .collect::<Vec<_>>(),
            vec![(1, 501), (2, 502), (4, 503), (5, 504)]
        );
    }

    fn pool_item_row(
        id: i64,
        pool_id: i64,
        robot_id: i64,
        total_score: f64,
        runs_done: i32,
    ) -> PoolItemRow {
        PoolItemRow {
            id,
            pool_id,
            robot_id,
            source_code: format!("mine({id});"),
            total_score,
            runs_done,
        }
    }

    fn mining_rally_queue_row(
        id: i64,
        mining_area_id: i64,
        robot_id: i64,
        user_id: i64,
        seconds_left: i32,
    ) -> MiningRallyQueueRow {
        MiningRallyQueueRow {
            id,
            mining_area_id,
            robot_id,
            user_id,
            rally_result_id: None,
            player_number: None,
            score: None,
            claimed: false,
            seconds_left,
        }
    }
}
