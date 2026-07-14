use crate::Response;
use crate::http::Request;
use crate::session;

pub(crate) fn query_i64(request: &Request, name: &str) -> Option<i64> {
    request
        .query
        .get(name)
        .or_else(|| request.form.get(name))
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
}

pub(crate) fn query_signed_i64(request: &Request, name: &str) -> Option<i64> {
    request
        .query
        .get(name)
        .or_else(|| request.form.get(name))
        .and_then(|value| value.parse::<i64>().ok())
}

pub(crate) fn request_user_id(request: &Request) -> Option<i64> {
    session::user_id_from_request(request)
}

pub(crate) fn login_redirect(request: &Request) -> Response {
    if let Some(return_to) = login_return_to_from_request(request) {
        Response::redirect(format!(
            "login?returnTo={}",
            encode_query_component(&return_to)
        ))
    } else {
        Response::redirect("login")
    }
}

pub(crate) fn login_return_to_from_request(request: &Request) -> Option<String> {
    let path = request.path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    let return_to = if request.query.is_empty() {
        path.to_string()
    } else {
        let mut pairs: Vec<_> = request.query.iter().collect();
        pairs.sort_by_key(|(left, _)| *left);
        let query = pairs
            .into_iter()
            .map(|(name, value)| {
                format!(
                    "{}={}",
                    encode_query_component(name),
                    encode_query_component(value)
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        format!("{path}?{query}")
    };
    if valid_login_return_to(&return_to).is_some() {
        Some(return_to)
    } else {
        None
    }
}

pub(crate) fn valid_login_return_to(value: &str) -> Option<&str> {
    if value.is_empty() || value.contains("://") || value.starts_with('/') {
        return None;
    }
    let path = value.split('?').next().unwrap_or(value);
    if path.eq_ignore_ascii_case("login") || path.eq_ignore_ascii_case("logoff") {
        return None;
    }
    Some(value)
}

pub(crate) fn encode_query_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub(crate) fn auth_page_href(signup: bool, return_to: Option<&str>) -> String {
    let mut href = String::from("login");
    let mut params = Vec::new();
    if signup {
        params.push("signup=1".to_string());
    }
    if let Some(return_to) = return_to {
        params.push(format!("returnTo={}", encode_query_component(return_to)));
    }
    if !params.is_empty() {
        href.push('?');
        href.push_str(&params.join("&"));
    }
    href
}

pub(crate) fn session_username(request: &Request) -> String {
    request
        .headers
        .get("cookie")
        .and_then(|cookies| session::cookie_value(cookies, "robominer_username"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Player".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::http::split_target;
    use crate::session::format_authenticated_cookie;

    use super::{
        auth_page_href, encode_query_component, login_redirect, login_return_to_from_request,
        query_i64, request_user_id, valid_login_return_to,
    };
    use crate::Request;

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

    #[test]
    fn query_parsing_decodes_parameters() {
        let request = request("/activity?rallyResultId=12&name=Robo+Miner%21");

        assert_eq!(request.path, "/activity");
        assert_eq!(query_i64(&request, "rallyResultId"), Some(12));
        assert_eq!(request.query.get("name"), Some(&"Robo Miner!".to_string()));
    }

    #[test]
    fn login_redirect_preserves_return_to_for_protected_routes() {
        let response = login_redirect(&request("/shop?selectedRobotPartTypeId=3"));
        assert_eq!(response.status, 302);
        assert!(response.headers.iter().any(|(name, value)| {
            *name == "Location" && value == "login?returnTo=shop%3FselectedRobotPartTypeId%3D3"
        }));
    }

    #[test]
    fn login_redirect_omits_return_to_for_root_and_auth_routes() {
        let root = login_redirect(&request("/"));
        assert!(
            root.headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "login")
        );

        let login = login_redirect(&request("/login"));
        assert!(
            login
                .headers
                .iter()
                .any(|(name, value)| *name == "Location" && value == "login")
        );
    }

    #[test]
    fn valid_login_return_to_rejects_external_and_auth_paths() {
        assert_eq!(
            valid_login_return_to("miningResults?rallyResultId=12"),
            Some("miningResults?rallyResultId=12")
        );
        assert_eq!(valid_login_return_to("https://evil.test"), None);
        assert_eq!(valid_login_return_to("/shop"), None);
        assert_eq!(valid_login_return_to("login"), None);
        assert_eq!(valid_login_return_to("login?returnTo=shop"), None);
    }

    #[test]
    fn login_return_to_from_request_builds_stable_query_strings() {
        assert_eq!(
            login_return_to_from_request(&request("/robot?robotId=2&tab=program")),
            Some("robot?robotId=2&tab=program".to_string())
        );
    }

    #[test]
    fn auth_page_href_preserves_signup_and_return_to() {
        assert_eq!(auth_page_href(false, None), "login");
        assert_eq!(
            auth_page_href(true, Some("shop?selectedRobotPartTypeId=3")),
            "login?signup=1&returnTo=shop%3FselectedRobotPartTypeId%3D3"
        );
    }

    #[test]
    fn encode_query_component_percent_encodes_spaces() {
        assert_eq!(encode_query_component("a b"), "a%20b");
    }

    #[test]
    fn user_id_is_read_from_signed_session_cookie_only() {
        assert_eq!(request_user_id(&request("/miningResults?userId=42")), None);
        assert_eq!(
            request_user_id(&request_with_cookie(
                "/miningResults",
                &format_authenticated_cookie(77, "Player")
            )),
            Some(77)
        );
    }
}
