//! On-disk static asset serving: path resolution, content-type sniffing, an
//! in-memory cache, and ETag / `If-None-Match` handling.

use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::http::{Request, Response};

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

pub(crate) async fn static_response(path: &str, static_root: &Path, request: &Request) -> Response {
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
    format!("\"{}\"", crate::static_assets::content_hash_hex(body))
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

    #[tokio::test(flavor = "current_thread")]
    async fn static_response_sets_cache_headers_and_honors_if_none_match() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
        let request = Request {
            method: "GET".to_string(),
            path: "/css/pages/layout_shell.css".to_string(),
            query: HashMap::new(),
            form: HashMap::new(),
            form_values: HashMap::new(),
            headers: HashMap::new(),
        };
        let response = static_response("/css/pages/layout_shell.css", &root, &request).await;
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
        let not_modified = static_response("/css/pages/layout_shell.css", &root, &cached).await;
        assert_eq!(not_modified.status, 304);
        assert!(not_modified.body.is_empty());
    }
}
