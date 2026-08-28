use crate::Request;
use crate::session::{self, session_clear_cookie_header};
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page, health,
    help_pages, leaderboard_page, login_redirect, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, request_user_id, robot_page, robot_stats_page,
    shop_page, static_files,
};
use std::borrow::Cow;

pub async fn route(request: &Request, config: &ServerConfig) -> Response {
    if matches!(request.path.as_str(), "/health" | "/Health")
        && matches!(request.method.as_str(), "GET" | "HEAD")
    {
        return health::health_response(config).await;
    }

    if let Some(response) = canonical_path_redirect(request) {
        return response;
    }

    let (session_strip, effective_request) = match config.database_pool.as_ref() {
        Some(pool) if session::session_from_request(request).is_some() => {
            let mut owned = request.clone();
            let strip = strip_stale_session_cookie(&mut owned, pool).await;
            (strip, Cow::Owned(owned))
        }
        Some(_) => (SessionStrip::Keep, Cow::Borrowed(request)),
        None => (SessionStrip::Keep, Cow::Borrowed(request)),
    };

    let mut response = dispatch(effective_request.as_ref(), config).await;
    if matches!(session_strip, SessionStrip::InvalidatePermanently) {
        response = clear_stale_session_cookies(response);
    }
    response
}

/// How to treat the request session after checking `User.sessionVersion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionStrip {
    /// Session matches DB (or no session cookie).
    Keep,
    /// Version mismatch / unknown user: strip request cookie and clear via Set-Cookie.
    InvalidatePermanently,
    /// DB lookup failed: strip request auth for this request only (do not clear cookie).
    TreatAsAnonymous,
}

/// Map a session-version lookup to a strip action. `Err` means the DB was unreachable.
fn session_strip_for_version_lookup(
    lookup: Result<Option<i32>, ()>,
    session_version: i32,
) -> SessionStrip {
    match lookup {
        Err(()) => SessionStrip::TreatAsAnonymous,
        Ok(current) if current == Some(session_version) => SessionStrip::Keep,
        Ok(_) => SessionStrip::InvalidatePermanently,
    }
}

async fn strip_stale_session_cookie(
    request: &mut Request,
    pool: &robominer_db::MySqlPool,
) -> SessionStrip {
    let Some(session) = session::session_from_request(request) else {
        return SessionStrip::Keep;
    };

    let lookup = match robominer_db::get_user_session_version(pool, session.user_id).await {
        Ok(version) => Ok(version),
        Err(error) => {
            tracing::warn!(
                user_id = session.user_id,
                error = %error,
                "session version lookup failed; treating request as anonymous"
            );
            Err(())
        }
    };
    let action = session_strip_for_version_lookup(lookup, session.session_version);
    if matches!(
        action,
        SessionStrip::InvalidatePermanently | SessionStrip::TreatAsAnonymous
    ) && let Some(cookies) = request.headers.get_mut("cookie")
    {
        *cookies = strip_named_cookie(cookies, "robominer_session");
    }
    action
}

fn strip_named_cookie(cookies: &str, name: &str) -> String {
    cookies
        .split(';')
        .filter_map(|cookie| {
            let trimmed = cookie.trim();
            if trimmed.is_empty() {
                return None;
            }
            let cookie_name = trimmed.split_once('=').map(|(n, _)| n).unwrap_or(trimmed);
            if cookie_name == name {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn clear_stale_session_cookies(response: Response) -> Response {
    response
        .with_header("Set-Cookie", session_clear_cookie_header())
        .with_header(
            "Set-Cookie",
            "robominer_user_id=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax",
        )
        .with_header("Set-Cookie", session::username_clear_cookie_header())
}

/// Redirect legacy PascalCase paths (`/Shop`, `/MiningQueue`, …) to canonical camelCase.
///
/// GET/HEAD only: mutating POSTs must reach the handler directly so form bodies and
/// CSRF tokens are not dropped by a redirect.
fn canonical_path_redirect(request: &Request) -> Option<Response> {
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return None;
    }

    let canonical = canonicalize_path(&request.path)?;
    if canonical == request.path {
        return None;
    }

    log_legacy_path_redirect(request, &canonical);

    let mut location = canonical;
    if !request.query.is_empty() {
        let mut pairs: Vec<_> = request.query.iter().collect();
        pairs.sort_by_key(|(key, _)| *key);
        location.push('?');
        for (index, (key, value)) in pairs.into_iter().enumerate() {
            if index > 0 {
                location.push('&');
            }
            location.push_str(key);
            location.push('=');
            location.push_str(value);
        }
    }

    Some(Response::redirect(location))
}

fn log_legacy_path_redirect(request: &Request, canonical: &str) {
    tracing::info!(
        legacy_path = %request.path,
        canonical_path = %canonical,
        method = %request.method,
        "legacy_pascal_case_redirect"
    );
}

fn canonicalize_path(path: &str) -> Option<String> {
    let rest = path.strip_prefix('/')?;
    let mut chars = rest.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    Some(format!("/{}{}", first.to_ascii_lowercase(), chars.as_str()))
}

async fn dispatch(request: &Request, config: &ServerConfig) -> Response {
    // Auth policy (by path family):
    // - Public: /health, /login|/signup|/logoff, /help*, /activity (read)
    // - Login required (+ CSRF on POST): shop, mining queue, robot, edit code,
    //   account, achievements, mining results, leaderboard (read), area overview
    // - Mining wallet claims: background worker via robominer-engine (rally rallies / mining claim-all)
    if !matches!(request.method.as_str(), "GET" | "HEAD" | "POST") {
        return Response::method_not_allowed();
    }

    match request.path.as_str() {
        "/" => {
            if request_user_id(request).is_some() {
                Response::redirect("miningQueue")
            } else {
                login_redirect(request)
            }
        }
        "/achievements" | "/Achievements" => {
            achievements_page::achievements_page(request, config).await
        }
        "/account" | "/Account" => account_page::account_page(request, config).await,
        "/activity" | "/Activity" => rally_pages::activity_page(request, config).await,
        "/editCode" | "/EditCode" => edit_code_page::edit_code_page(request, config).await,
        "/help" | "/Help" => {
            help_pages::help_page(request, config, request.query.contains_key("welcome")).await
        }
        "/helpTutorial" | "/help_tutorial.html" => {
            help_pages::help_text_page(request, config, "helpTutorial", query_i64(request, "step"))
                .await
        }
        "/helpProgramTips" | "/help_programtips.html" => {
            help_pages::help_text_page(request, config, "helpProgramTips", None).await
        }
        "/helpRobotProgram" | "/help_robotprogram.html" => {
            help_pages::help_text_page(request, config, "helpRobotProgram", None).await
        }
        "/helpMechanics" | "/help_mechanics.html" => {
            help_pages::help_text_page(request, config, "helpMechanics", None).await
        }
        "/leaderboard" | "/Leaderboard" => {
            leaderboard_page::leaderboard_page(request, config).await
        }
        "/login" | "/Login" => auth_pages::login_page(request, config).await,
        "/logoff" | "/Logoff" => auth_pages::logoff_page(request),
        "/miningQueue" | "/MiningQueue" => {
            mining_queue_page::mining_queue_page(request, config).await
        }
        "/miningResults" | "/MiningResults" => {
            mining_results_page::mining_results_page(request, config).await
        }
        "/miningAreaOverview" | "/MiningAreaOverview" => {
            mining_area_overview_page::mining_area_overview_page(request, config).await
        }
        "/robot" | "/Robot" => robot_page::robot_page(request, config).await,
        "/robotStats" | "/RobotStats" => robot_stats_page::robot_stats_page(request, config).await,
        "/shop" | "/Shop" => shop_page::shop_page(request, config).await,
        _ => static_files::static_response(&request.path, &config.static_root, request).await,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use crate::http::split_target;
    use crate::session::format_authenticated_cookie;
    use crate::static_files::static_file_path;
    use crate::{Request, Response, ServerConfig};

    use super::{SessionStrip, canonicalize_path, route, session_strip_for_version_lookup};

    #[test]
    fn session_strip_keeps_matching_version() {
        assert_eq!(
            session_strip_for_version_lookup(Ok(Some(3)), 3),
            SessionStrip::Keep
        );
    }

    #[test]
    fn session_strip_invalidates_mismatched_or_missing_version() {
        assert_eq!(
            session_strip_for_version_lookup(Ok(Some(4)), 3),
            SessionStrip::InvalidatePermanently
        );
        assert_eq!(
            session_strip_for_version_lookup(Ok(None), 0),
            SessionStrip::InvalidatePermanently
        );
    }

    #[test]
    fn session_strip_treats_db_errors_as_anonymous_without_permanent_clear() {
        assert_eq!(
            session_strip_for_version_lookup(Err(()), 1),
            SessionStrip::TreatAsAnonymous
        );
    }

    fn request(path: &str) -> Request {
        let (path, query) = split_target(path);
        Request {
            method: "GET".to_string(),
            path,
            query,
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    fn request_with_cookie(path: &str, cookie: &str) -> Request {
        let mut request = request(path);
        request
            .headers
            .insert("cookie".to_string(), cookie.to_string());
        request
    }

    fn authenticated_request(path: &str) -> Request {
        request_with_cookie(path, &format_authenticated_cookie(42, "Player"))
    }

    fn assert_login_redirect(response: &Response, expected_location: &str) {
        assert_eq!(response.status, 302);
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == expected_location)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn health_route_is_public_without_database() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let response = route(&request("/health"), &config).await;

        assert_eq!(response.status, 200);
        let body = String::from_utf8_lossy(&response.body);
        assert!(body.contains("database=unconfigured"), "body={body}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn root_route_redirects_to_login_when_logged_out() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let response = route(&request("/"), &config).await;

        assert_login_redirect(&response, "login");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn root_route_redirects_to_mining_queue_when_logged_in() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let response = route(&authenticated_request("/"), &config).await;

        assert_eq!(response.status, 302);
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "miningQueue")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn protected_routes_redirect_to_login_when_logged_out() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        for path in [
            "/account",
            "/achievements",
            "/editCode",
            "/miningQueue",
            "/miningResults",
            "/robot",
            "/robotStats",
            "/shop",
        ] {
            let response = route(&request(path), &config).await;
            let expected = format!("login?returnTo={}", path.trim_start_matches('/'));
            assert_login_redirect(&response, &expected);
        }
    }

    #[test]
    fn static_paths_cannot_escape_web_root() {
        assert!(static_file_path("/../Cargo.toml", Path::new("robominer-web/static")).is_none());
        assert!(
            static_file_path("/css/../robominer.css", Path::new("robominer-web/static")).is_none()
        );
    }

    #[test]
    fn canonicalize_path_lowercases_leading_letter_only() {
        assert_eq!(canonicalize_path("/Shop").as_deref(), Some("/shop"));
        assert_eq!(
            canonicalize_path("/MiningQueue").as_deref(),
            Some("/miningQueue")
        );
        assert_eq!(canonicalize_path("/shop"), None);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn legacy_pascal_case_paths_redirect_to_canonical_routes() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let cases = [
            ("/Shop", "/shop"),
            ("/MiningQueue", "/miningQueue"),
            ("/EditCode", "/editCode"),
            ("/Account", "/account"),
            ("/Robot", "/robot"),
            ("/Achievements", "/achievements"),
        ];
        for (legacy, canonical) in cases {
            let response = route(&request(legacy), &config).await;
            assert_eq!(response.status, 302, "{legacy} should redirect");
            assert!(
                response
                    .headers
                    .iter()
                    .any(|(name, value)| *name == "Location" && value == canonical),
                "{legacy} should redirect to {canonical}, got {:?}",
                response.headers
            );
        }

        let mut with_query = request("/MiningQueue");
        with_query
            .query
            .insert("fragment".to_string(), "queue".to_string());
        with_query.query.insert("info".to_string(), "1".to_string());
        let response = route(&with_query, &config).await;
        assert_eq!(response.status, 302);
        let location = response
            .headers
            .iter()
            .find(|(name, _)| *name == "Location")
            .map(|(_, value)| value.as_str())
            .expect("Location header");
        assert!(
            location.starts_with("/miningQueue?"),
            "query redirect should keep canonical path: {location}"
        );
        assert!(
            location.contains("fragment=queue") && location.contains("info=1"),
            "query string should be preserved: {location}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn legacy_pascal_case_post_requests_are_not_redirected() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        for (path, forbidden_location) in [("/Login", "/login"), ("/Shop", "/shop")] {
            let mut request = request(path);
            request.method = "POST".to_string();
            request
                .form
                .insert("loginName".to_string(), "player".to_string());
            request
                .form
                .insert("password".to_string(), "secret".to_string());

            let response = route(&request, &config).await;
            let location = response
                .headers
                .iter()
                .find(|(name, _)| *name == "Location")
                .map(|(_, value)| value.as_str());
            assert_ne!(
                location,
                Some(forbidden_location),
                "POST {path} must not canonical-redirect to {forbidden_location} (got status {}, location {:?})",
                response.status,
                location
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn get_logoff_does_not_clear_session_cookie() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let response = route(&authenticated_request("/logoff"), &config).await;
        let cookie_headers: Vec<_> = response
            .headers
            .iter()
            .filter(|(name, _)| *name == "Set-Cookie")
            .map(|(_, value)| value.as_str())
            .collect();

        assert_eq!(response.status, 200);
        assert!(
            !cookie_headers
                .iter()
                .any(|header| header.starts_with("robominer_session=; Max-Age=0;")),
            "GET /logoff must not clear the session cookie"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn post_logoff_without_csrf_is_forbidden_when_authenticated() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let mut request = authenticated_request("/logoff");
        request.method = "POST".to_string();
        let response = route(&request, &config).await;
        assert_eq!(response.status, 403);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn post_logoff_with_csrf_clears_session_cookie() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let cookie = format_authenticated_cookie(42, "Player");
        let token = crate::csrf::csrf_token_from_cookie(&cookie).expect("csrf token");
        let mut request = request_with_cookie("/logoff", &cookie);
        request.method = "POST".to_string();
        request
            .form
            .insert(crate::csrf::CSRF_FIELD_NAME.to_string(), token);

        let response = route(&request, &config).await;
        let cookie_headers: Vec<_> = response
            .headers
            .iter()
            .filter(|(name, _)| *name == "Set-Cookie")
            .map(|(_, value)| value.as_str())
            .collect();

        assert_eq!(response.status, 200);
        assert!(
            cookie_headers
                .iter()
                .any(|header| header.starts_with("robominer_session=; Max-Age=0;"))
        );
    }
}
