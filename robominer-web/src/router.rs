use crate::Request;
use crate::session::{self, session_clear_cookie_header};
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page, health,
    help_page, http, leaderboard_page, login_redirect, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, request_user_id, robot_page, shop_page,
};

pub async fn route(request: &Request, config: &ServerConfig) -> Response {
    if matches!(request.path.as_str(), "/health" | "/Health")
        && matches!(request.method.as_str(), "GET" | "HEAD")
    {
        return health::health_response(config).await;
    }

    let mut request = request.clone();
    let clear_stale_session = match config.database_pool.as_ref() {
        Some(pool) => strip_stale_session_cookie(&mut request, pool).await,
        None => false,
    };

    let mut response = dispatch(&request, config).await;
    if clear_stale_session {
        response = clear_stale_session_cookies(response);
    }
    response
}

async fn strip_stale_session_cookie(
    request: &mut Request,
    pool: &robominer_db::MySqlPool,
) -> bool {
    let Some(session) = session::session_from_request(request) else {
        return false;
    };

    let Ok(current_version) = robominer_db::get_user_session_version(pool, session.user_id).await
    else {
        // Fail open on transient DB errors so a blip does not mass-log everyone out.
        return false;
    };

    let session_valid = current_version == Some(session.session_version);
    if session_valid {
        return false;
    }

    if let Some(cookies) = request.headers.get_mut("cookie") {
        *cookies = strip_named_cookie(cookies, "robominer_session");
    }
    true
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
        .with_header(
            "Set-Cookie",
            "robominer_username=; Max-Age=0; Path=/; SameSite=Lax",
        )
}

async fn dispatch(request: &Request, config: &ServerConfig) -> Response {
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
            help_page::help_page(request, config, request.query.contains_key("welcome")).await
        }
        "/helpTutorial" | "/help_tutorial.html" => {
            help_page::help_text_page(request, config, "helpTutorial", query_i64(request, "step"))
                .await
        }
        "/helpProgramTips" | "/help_programtips.html" => {
            help_page::help_text_page(request, config, "helpProgramTips", None).await
        }
        "/helpRobotProgram" | "/help_robotprogram.html" => {
            help_page::help_text_page(request, config, "helpRobotProgram", None).await
        }
        "/helpMechanics" | "/help_mechanics.html" => {
            help_page::help_text_page(request, config, "helpMechanics", None).await
        }
        "/leaderboard" | "/Leaderboard" => {
            leaderboard_page::leaderboard_page(request, config).await
        }
        "/login" | "/Login" => auth_pages::login_page(request, config).await,
        "/logoff" | "/Logoff" => auth_pages::logoff_page(),
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
        "/shop" | "/Shop" => shop_page::shop_page(request, config).await,
        _ => http::static_response(&request.path, &config.static_root, request).await,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use crate::http::{split_target, static_file_path};
    use crate::session::format_authenticated_cookie;
    use crate::{Request, Response, ServerConfig};

    use super::route;

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

    #[tokio::test(flavor = "current_thread")]
    async fn logoff_route_clears_session_cookie() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
            trust_proxy: false,
        };

        let response = route(&request("/logoff"), &config).await;
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
