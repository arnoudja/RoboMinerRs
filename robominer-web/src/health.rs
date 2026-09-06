use std::sync::Arc;

use crate::ServerConfig;
use crate::http::Response;

/// Process liveness: always 200 if the HTTP server can answer.
pub async fn live_response(_config: &ServerConfig) -> Response {
    plain(200, "OK", "ok\nliveness=ok\n")
}

/// Readiness: database configured, reachable, and migrations current.
///
/// Unlike [`health_response`], an unconfigured database is **not** ready (503).
pub async fn ready_response(config: &ServerConfig) -> Response {
    match config.database_pool.as_ref() {
        None => plain(503, "Service Unavailable", "unavailable\n"),
        Some(pool) => match check_database(pool).await {
            Ok(()) => plain(200, "OK", "ok\ndatabase=ok\nmigrations=ok\n"),
            Err(()) => plain(503, "Service Unavailable", "unavailable\n"),
        },
    }
}

/// Combined probe kept for systemd / existing monitors.
///
/// - No database configured: process is up → 200 (`database=unconfigured`).
/// - Database configured: requires a live ping and all embedded migrations
///   applied → otherwise 503 with an opaque body (details stay in server logs).
///
/// Prefer [`live_response`] / [`ready_response`] for new orchestrator probes.
pub async fn health_response(config: &ServerConfig) -> Response {
    match config.database_pool.as_ref() {
        None => plain(200, "OK", "ok\ndatabase=unconfigured\nmigrations=skipped\n"),
        Some(pool) => match check_database(pool).await {
            Ok(()) => plain(200, "OK", "ok\ndatabase=ok\nmigrations=ok\n"),
            Err(()) => plain(503, "Service Unavailable", "unavailable\n"),
        },
    }
}

async fn check_database(pool: &robominer_db::MySqlPool) -> Result<(), ()> {
    if let Err(error) = robominer_db::ping(pool).await {
        tracing::error!(%error, "health_check_database_ping_failed");
        return Err(());
    }

    let status = match robominer_db::migration_status(pool, robominer_db::EMBEDDED_MIGRATIONS).await
    {
        Ok(status) => status,
        Err(error) => {
            tracing::error!(%error, "health_check_migration_status_failed");
            return Err(());
        }
    };

    let pending: Vec<&str> = status
        .iter()
        .filter(|(_, applied)| !*applied)
        .map(|(version, _)| version.as_str())
        .collect();

    if pending.is_empty() {
        Ok(())
    } else {
        tracing::error!(
            pending = %pending.join(","),
            "health_check_migrations_pending"
        );
        Err(())
    }
}

fn plain(status: u16, reason: &'static str, body: &str) -> Response {
    Response {
        status,
        reason,
        content_type: "text/plain; charset=utf-8",
        headers: vec![("Cache-Control", "no-store".to_string())],
        body: Arc::from(body.as_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{health_response, live_response, ready_response};
    use crate::ServerConfig;

    fn config_without_db() -> ServerConfig {
        ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: false,
            trust_proxy: false,
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn health_without_database_reports_unconfigured() {
        let response = health_response(&config_without_db()).await;
        assert_eq!(response.status, 200);
        let body = String::from_utf8_lossy(&response.body);
        assert!(body.starts_with("ok\n"), "body={body}");
        assert!(body.contains("database=unconfigured"), "body={body}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn live_without_database_is_ok() {
        let response = live_response(&config_without_db()).await;
        assert_eq!(response.status, 200);
        let body = String::from_utf8_lossy(&response.body);
        assert!(body.contains("liveness=ok"), "body={body}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ready_without_database_is_unavailable() {
        let response = ready_response(&config_without_db()).await;
        assert_eq!(response.status, 503);
    }
}
