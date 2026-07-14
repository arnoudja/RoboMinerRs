use crate::Request;
use crate::{
    Response, ServerConfig, account_page, achievements_page, auth_pages, edit_code_page, help_page,
    http, leaderboard_page, login_redirect, mining_area_overview_page, mining_queue_page,
    mining_results_page, query_i64, rally_pages, request_user_id, robot_page, shop_page,
};

pub fn route(request: &Request, config: &ServerConfig) -> Response {
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
        "/achievements" | "/Achievements" => achievements_page::achievements_page(request, config),
        "/account" | "/Account" => account_page::account_page(request, config),
        "/activity" | "/Activity" => rally_pages::activity_page(request, config),
        "/editCode" | "/EditCode" => edit_code_page::edit_code_page(request, config),
        "/help" | "/Help" => {
            help_page::help_page(request, config, request.query.contains_key("welcome"))
        }
        "/helpTutorial" | "/help_tutorial.html" => {
            help_page::help_text_page(request, config, "helpTutorial", query_i64(request, "step"))
        }
        "/helpProgramTips" | "/help_programtips.html" => {
            help_page::help_text_page(request, config, "helpProgramTips", None)
        }
        "/helpRobotProgram" | "/help_robotprogram.html" => {
            help_page::help_text_page(request, config, "helpRobotProgram", None)
        }
        "/helpMechanics" | "/help_mechanics.html" => {
            help_page::help_text_page(request, config, "helpMechanics", None)
        }
        "/leaderboard" | "/Leaderboard" => leaderboard_page::leaderboard_page(request, config),
        "/login" | "/Login" => auth_pages::login_page(request, config),
        "/logoff" | "/Logoff" => auth_pages::logoff_page(),
        "/miningQueue" | "/MiningQueue" => mining_queue_page::mining_queue_page(request, config),
        "/miningResults" | "/MiningResults" => {
            mining_results_page::mining_results_page(request, config)
        }
        "/miningAreaOverview" | "/MiningAreaOverview" => {
            mining_area_overview_page::mining_area_overview_page(request, config)
        }
        "/robot" | "/Robot" => robot_page::robot_page(request, config),
        "/shop" | "/Shop" => shop_page::shop_page(request, config),
        _ => http::static_response(&request.path, &config.static_root),
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

    #[test]
    fn root_route_redirects_to_login_when_logged_out() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
        };

        let response = route(&request("/"), &config);

        assert_login_redirect(&response, "login");
    }

    #[test]
    fn root_route_redirects_to_mining_queue_when_logged_in() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
        };

        let response = route(&authenticated_request("/"), &config);

        assert_eq!(response.status, 302);
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "miningQueue")
        );
    }

    #[test]
    fn protected_routes_redirect_to_login_when_logged_out() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
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
            let response = route(&request(path), &config);
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
    fn logoff_route_clears_session_cookie() {
        let config = ServerConfig {
            static_root: PathBuf::from("robominer-web/static"),
            database_pool: None,
            allow_signup: true,
        };

        let response = route(&request("/logoff"), &config);
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
