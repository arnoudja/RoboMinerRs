use std::collections::HashMap;
use std::sync::Arc;

/// Upper bound for HTTP request bodies (program saves are the largest forms).
pub(crate) const MAX_REQUEST_BODY_BYTES: usize = 1_048_576;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub form: HashMap<String, String>,
    pub form_values: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub reason: &'static str,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: Arc<[u8]>,
}

impl Response {
    pub(crate) fn html(body: String) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "text/html; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(body.into_bytes()),
        }
    }

    pub(crate) fn redirect(location: impl Into<String>) -> Self {
        Self {
            status: 302,
            reason: "Found",
            content_type: "text/plain; charset=utf-8",
            headers: vec![("Location", location.into())],
            body: Arc::from([] as [u8; 0]),
        }
    }

    pub(crate) fn not_found() -> Self {
        Self {
            status: 404,
            reason: "Not Found",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(*b"Not found"),
        }
    }

    pub(crate) fn method_not_allowed() -> Self {
        Self {
            status: 405,
            reason: "Method Not Allowed",
            content_type: "text/plain; charset=utf-8",
            headers: vec![("Allow", "GET, HEAD, POST".to_string())],
            body: Arc::from(*b"Method not allowed"),
        }
    }

    pub(crate) fn payload_too_large() -> Self {
        Self {
            status: 413,
            reason: "Payload Too Large",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(*b"Request body too large"),
        }
    }

    pub(crate) fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: 403,
            reason: "Forbidden",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(message.into().into_bytes()),
        }
    }

    pub(crate) fn too_many_requests(message: impl Into<String>) -> Self {
        Self {
            status: 429,
            reason: "Too Many Requests",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(message.into().into_bytes()),
        }
    }

    pub(crate) fn internal_error() -> Self {
        Self {
            status: 500,
            reason: "Internal Server Error",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(*b"Internal server error"),
        }
    }

    pub(crate) fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: 503,
            reason: "Service Unavailable",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: Arc::from(message.into().into_bytes()),
        }
    }

    pub(crate) fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }
}

#[cfg(test)]
impl Response {
    pub(crate) fn body_utf8(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

pub(crate) fn split_target(target: &str) -> (String, HashMap<String, String>) {
    let (path, query_string) = target.split_once('?').unwrap_or((target, ""));
    let mut query = HashMap::new();

    for pair in query_string.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        query.insert(percent_decode(name), percent_decode(value));
    }

    (path.to_string(), query)
}

fn percent_decode(value: &str) -> String {
    let mut result = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                result.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                    result.push(hex);
                    index += 3;
                } else {
                    result.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                result.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&result).into_owned()
}

pub(crate) fn parse_form_body_values(
    headers: &HashMap<String, String>,
    body: &[u8],
) -> HashMap<String, Vec<String>> {
    let Some(content_type) = headers.get("content-type") else {
        return HashMap::new();
    };
    if !content_type
        .split(';')
        .next()
        .is_some_and(|value| value.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
    {
        return HashMap::new();
    }

    let body = String::from_utf8_lossy(body);
    split_form_field_values(&body)
}

pub(crate) fn split_form_field_values(fields: &str) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for pair in fields.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        result
            .entry(percent_decode(name))
            .or_insert_with(Vec::new)
            .push(percent_decode(value));
    }
    result
}

pub(crate) fn first_form_values(values: &HashMap<String, Vec<String>>) -> HashMap<String, String> {
    values
        .iter()
        .filter_map(|(name, values)| values.first().map(|value| (name.clone(), value.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_too_large_response_uses_413() {
        let response = Response::payload_too_large();
        assert_eq!(response.status, 413);
        assert_eq!(response.reason, "Payload Too Large");
    }

    #[test]
    fn response_builders_cover_common_error_statuses() {
        let method = Response::method_not_allowed();
        assert_eq!(method.status, 405);
        assert_eq!(method.reason, "Method Not Allowed");
        assert_eq!(
            method.headers,
            vec![("Allow", "GET, HEAD, POST".to_string())]
        );

        let internal = Response::internal_error();
        assert_eq!(internal.status, 500);
        assert_eq!(internal.reason, "Internal Server Error");
        assert_eq!(
            String::from_utf8_lossy(&internal.body),
            "Internal server error"
        );
    }

    #[test]
    fn percent_decode_handles_plus_and_invalid_escapes() {
        let (path, query) = split_target("/page?name=hello+world&bad=%ZZ&cut=%A&ok=%2F");
        assert_eq!(path, "/page");
        assert_eq!(query.get("name").map(String::as_str), Some("hello world"));
        assert_eq!(query.get("bad").map(String::as_str), Some("%ZZ"));
        assert_eq!(query.get("cut").map(String::as_str), Some("%A"));
        assert_eq!(query.get("ok").map(String::as_str), Some("/"));
    }

    #[test]
    fn parse_form_body_values_only_accepts_urlencoded_content_type() {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        let values = parse_form_body_values(&headers, b"a=1&a=2&b=hello+world");
        assert_eq!(
            values.get("a"),
            Some(&vec!["1".to_string(), "2".to_string()])
        );
        assert_eq!(values.get("b"), Some(&vec!["hello world".to_string()]));

        headers.insert("content-type".to_string(), "text/plain".to_string());
        assert!(parse_form_body_values(&headers, b"a=1").is_empty());
    }
}
