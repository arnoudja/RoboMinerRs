use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use sha2::{Digest, Sha256};

/// Upper bound for HTTP request bodies (program saves are the largest forms).
pub(crate) const MAX_REQUEST_BODY_BYTES: usize = 1_048_576;
const STATIC_CACHE_CONTROL: &str = "public, max-age=604800";

#[derive(Clone)]
struct StaticFileEntry {
    body: Arc<[u8]>,
    etag: String,
    content_type: &'static str,
}

fn static_file_cache() -> &'static Mutex<HashMap<PathBuf, StaticFileEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, StaticFileEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub form: HashMap<String, String>,
    pub form_values: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub reason: &'static str,
    pub content_type: &'static str,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub(crate) fn html(body: String) -> Self {
        Self {
            status: 200,
            reason: "OK",
            content_type: "text/html; charset=utf-8",
            headers: Vec::new(),
            body: body.into_bytes(),
        }
    }

    pub(crate) fn redirect(location: impl Into<String>) -> Self {
        Self {
            status: 302,
            reason: "Found",
            content_type: "text/plain; charset=utf-8",
            headers: vec![("Location", location.into())],
            body: Vec::new(),
        }
    }

    pub(crate) fn not_found() -> Self {
        Self {
            status: 404,
            reason: "Not Found",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: b"Not found".to_vec(),
        }
    }

    pub(crate) fn method_not_allowed() -> Self {
        Self {
            status: 405,
            reason: "Method Not Allowed",
            content_type: "text/plain; charset=utf-8",
            headers: vec![("Allow", "GET, HEAD, POST".to_string())],
            body: b"Method not allowed".to_vec(),
        }
    }

    pub(crate) fn payload_too_large() -> Self {
        Self {
            status: 413,
            reason: "Payload Too Large",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: b"Request body too large".to_vec(),
        }
    }

    pub(crate) fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: 403,
            reason: "Forbidden",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: message.into().into_bytes(),
        }
    }

    pub(crate) fn too_many_requests(message: impl Into<String>) -> Self {
        Self {
            status: 429,
            reason: "Too Many Requests",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: message.into().into_bytes(),
        }
    }

    pub(crate) fn internal_error() -> Self {
        Self {
            status: 500,
            reason: "Internal Server Error",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: b"Internal server error".to_vec(),
        }
    }

    pub(crate) fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: 503,
            reason: "Service Unavailable",
            content_type: "text/plain; charset=utf-8",
            headers: Vec::new(),
            body: message.into().into_bytes(),
        }
    }

    pub(crate) fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
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

pub(crate) async fn static_response(
    path: &str,
    static_root: &Path,
    request: &Request,
) -> Response {
    let Some(file_path) = static_file_path(path, static_root) else {
        return Response::not_found();
    };

    let entry = match load_static_file_entry(file_path).await {
        Ok(Some(entry)) => entry,
        Ok(None) => return Response::not_found(),
        Err(_) => return Response::internal_error(),
    };

    if request
        .headers
        .get("if-none-match")
        .is_some_and(|value| etag_matches(value, &entry.etag))
    {
        return Response {
            status: 304,
            reason: "Not Modified",
            content_type: entry.content_type,
            headers: vec![
                ("ETag", entry.etag.clone()),
                ("Cache-Control", STATIC_CACHE_CONTROL.to_string()),
            ],
            body: Vec::new(),
        };
    }

    Response {
        status: 200,
        reason: "OK",
        content_type: entry.content_type,
        headers: vec![
            ("ETag", entry.etag.clone()),
            ("Cache-Control", STATIC_CACHE_CONTROL.to_string()),
        ],
        body: entry.body.to_vec(),
    }
}

async fn load_static_file_entry(file_path: PathBuf) -> Result<Option<StaticFileEntry>, ()> {
    if let Some(entry) = static_file_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&file_path)
        .cloned()
    {
        return Ok(Some(entry));
    }

    let cache_key = file_path.clone();
    match tokio::task::spawn_blocking(move || read_static_file_entry(file_path))
        .await
        .map_err(|_| ())?
    {
        Ok(entry) => {
            static_file_cache()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(cache_key, entry.clone());
            Ok(Some(entry))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(()),
    }
}

fn read_static_file_entry(file_path: PathBuf) -> std::io::Result<StaticFileEntry> {
    let body = fs::read(&file_path)?;
    let content_type = content_type(&file_path);
    let etag = static_etag(&body);
    Ok(StaticFileEntry {
        body: body.into(),
        etag,
        content_type,
    })
}

fn static_etag(body: &[u8]) -> String {
    let digest = Sha256::digest(body);
    let mut hex = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("\"{hex}\"")
}

fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    if_none_match
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate == etag)
}

pub(crate) fn static_file_path(path: &str, static_root: &Path) -> Option<PathBuf> {
    let relative = path.trim_start_matches('/');
    let mut file_path = static_root.to_path_buf();

    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => file_path.push(part),
            _ => return None,
        }
    }

    Some(file_path)
}

pub(crate) fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
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

    #[tokio::test(flavor = "current_thread")]
    async fn static_response_sets_cache_headers_and_honors_if_none_match() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
        let request = Request {
            method: "GET".to_string(),
            path: "/css/robominer.css".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        };
        let response = static_response("/css/robominer.css", &root, &request).await;
        assert_eq!(response.status, 200);
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| *name == "Cache-Control" && value == STATIC_CACHE_CONTROL)
        );
        let etag = response
            .headers
            .iter()
            .find(|(name, _)| *name == "ETag")
            .map(|(_, value)| value.clone())
            .expect("etag");

        let mut cached = request;
        cached.headers.insert("if-none-match".to_string(), etag);
        let not_modified = static_response("/css/robominer.css", &root, &cached).await;
        assert_eq!(not_modified.status, 304);
        assert!(not_modified.body.is_empty());
    }
}
