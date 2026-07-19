use crate::ServerConfig;
use crate::http::Response;

/// Liveness / readiness probe for loopback and reverse-proxy checks.
///
/// - No database configured: process is up → 200 (`database=unconfigured`).
/// - Database configured: requires a live ping and all embedded migrations
///   applied → otherwise 503.
pub async fn health_response(config: &ServerConfig) -> Response {
    match config.database_pool.as_ref() {
        None => plain(200, "OK", "ok\ndatabase=unconfigured\nmigrations=skipped\n"),
        Some(pool) => match check_database(pool).await {
            Ok(()) => plain(200, "OK", "ok\ndatabase=ok\nmigrations=ok\n"),
            Err(detail) => plain(
                503,
                "Service Unavailable",
                &format!("unavailable\n{detail}\n"),
            ),
        },
    }
}

async fn check_database(pool: &robominer_db::MySqlPool) -> Result<(), String> {
    robominer_db::ping(pool)
        .await
        .map_err(|error| format!("database=error\nerror={error}"))?;

    let status = robominer_db::migration_status(pool, robominer_db::EMBEDDED_MIGRATIONS)
        .await
        .map_err(|error| format!("database=ok\nmigrations=error\nerror={error}"))?;

    let pending: Vec<&str> = status
        .iter()
        .filter(|(_, applied)| !*applied)
        .map(|(version, _)| version.as_str())
        .collect();

    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "database=ok\nmigrations=pending\npending={}",
            pending.join(",")
        ))
    }
}

fn plain(status: u16, reason: &'static str, body: &str) -> Response {
    Response {
        status,
        reason,
        content_type: "text/plain; charset=utf-8",
        headers: vec![("Cache-Control", "no-store".to_string())],
        body: body.as_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::health_response;
    use crate::ServerConfig;

    #[tokio::test(flavor = "current_thread")]
    async fn health_without_database_reports_unconfigured() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: false,
            trust_proxy: false,
        };

        let response = health_response(&config).await;
        assert_eq!(response.status, 200);
        let body = String::from_utf8_lossy(&response.body);
        assert!(body.starts_with("ok\n"), "body={body}");
        assert!(body.contains("database=unconfigured"), "body={body}");
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Cache-Control" && value == "no-store")
        );
    }
}
