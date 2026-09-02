//! Login redirects, return-to URLs, and session identity from cookies.

use crate::Response;
use crate::http::Request;
use crate::session;

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
    if value.is_empty()
        || value.contains("://")
        || value.starts_with("//")
        || value.starts_with('/')
        || value.contains('\\')
    {
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
