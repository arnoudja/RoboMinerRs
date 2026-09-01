use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::http::split_target;
use crate::routes::{AppRoute, RoutePolicy};
use crate::session::format_authenticated_cookie;
use crate::static_files::static_file_path;
use crate::{Request, Response, ServerConfig};

use super::redirect::canonicalize_path;
use super::route;
use super::session_gate::{SessionStrip, session_strip_for_version_lookup};

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

#[test]
fn every_app_route_has_documented_policy() {
    for route in AppRoute::ALL {
        let policy = route.policy();
        match route {
            AppRoute::Login
            | AppRoute::Logoff
            | AppRoute::Help
            | AppRoute::HelpTutorial
            | AppRoute::HelpProgramTips
            | AppRoute::HelpRobotProgram
            | AppRoute::HelpMechanics => {
                assert_eq!(policy, RoutePolicy::Public, "{route:?}");
            }
            AppRoute::Activity | AppRoute::Leaderboard => {
                assert_eq!(policy, RoutePolicy::PublicRead, "{route:?}");
            }
            AppRoute::Achievements
            | AppRoute::Account
            | AppRoute::EditCode
            | AppRoute::MiningQueue
            | AppRoute::MiningResults
            | AppRoute::MiningAreaOverview
            | AppRoute::Robot
            | AppRoute::RobotStats
            | AppRoute::Shop => {
                assert_eq!(
                    policy,
                    RoutePolicy::SessionRequired { csrf_on_post: true },
                    "{route:?}"
                );
            }
        }
    }
    assert_eq!(
        AppRoute::ALL.len(),
        18,
        "update policy groups when adding routes"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn session_required_routes_redirect_when_logged_out() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    for app_route in AppRoute::ALL {
        if !matches!(
            app_route.policy(),
            RoutePolicy::SessionRequired { csrf_on_post: true }
        ) {
            continue;
        }
        let path = app_route.path();
        let response = route(&request(path), &config).await;
        let expected = format!("login?returnTo={}", path.trim_start_matches('/'));
        assert_login_redirect(&response, &expected);
    }
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
        "/miningAreaOverview",
        "/robot",
        "/robotStats",
        "/shop",
    ] {
        let response = route(&request(path), &config).await;
        let expected = format!("login?returnTo={}", path.trim_start_matches('/'));
        assert_login_redirect(&response, &expected);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn public_read_routes_do_not_require_login() {
    let config = ServerConfig {
        static_root: PathBuf::from("robominer-web/static"),
        database_pool: None,
        allow_signup: true,
        trust_proxy: false,
    };

    for path in ["/leaderboard", "/activity"] {
        let response = route(&request(path), &config).await;
        assert_ne!(
            response.status, 302,
            "{path} should not redirect to login when logged out"
        );
        assert_eq!(
            response.status, 503,
            "{path} without DB should return service unavailable"
        );
    }
}

#[test]
fn static_paths_cannot_escape_web_root() {
    assert!(static_file_path("/../Cargo.toml", Path::new("robominer-web/static")).is_none());
    assert!(static_file_path("/css/../robominer.css", Path::new("robominer-web/static")).is_none());
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
