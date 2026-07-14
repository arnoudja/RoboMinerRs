use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;

use crate::ServerConfig;

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

pub fn serve(listener: TcpListener, config: ServerConfig) -> std::io::Result<()> {
    for stream in listener.incoming() {
        let config = config.clone();
        let stream = stream?;
        thread::spawn(move || {
            if let Err(error) = handle_connection(stream, &config) {
                eprintln!("request failed: {error}");
            }
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, config: &ServerConfig) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let Some((method, target)) = parse_request_line(&request_line) else {
        write_response(&mut stream, &Response::not_found(), false)?;
        return Ok(());
    };
    let (path, query) = split_target(target);

    let headers = read_headers(&mut reader)?;
    let body = read_body(&mut reader, &headers)?;
    let form_values = parse_form_body_values(&headers, &body);
    let form = first_form_values(&form_values);

    let request = Request {
        method: method.to_string(),
        path,
        query,
        form,
        form_values,
        headers,
    };
    let head_only = request.method == "HEAD";
    let response = crate::route(&request, config);

    write_response(&mut stream, &response, head_only)
}

pub(crate) fn parse_request_line(request_line: &str) -> Option<(&str, &str)> {
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    Some((method, target))
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

fn read_headers(reader: &mut BufReader<TcpStream>) -> std::io::Result<HashMap<String, String>> {
    let mut headers = HashMap::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Ok(headers)
}

fn read_body(
    reader: &mut BufReader<TcpStream>,
    headers: &HashMap<String, String>,
) -> std::io::Result<Vec<u8>> {
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length == 0 {
        return Ok(Vec::new());
    }

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok(body)
}

fn parse_form_body_values(
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

pub(crate) fn static_response(path: &str, static_root: &Path) -> Response {
    let Some(file_path) = static_file_path(path, static_root) else {
        return Response::not_found();
    };

    match fs::read(&file_path) {
        Ok(body) => Response {
            status: 200,
            reason: "OK",
            content_type: content_type(&file_path),
            headers: Vec::new(),
            body,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Response::not_found(),
        Err(_) => Response::internal_error(),
    }
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

fn write_response(
    stream: &mut TcpStream,
    response: &Response,
    head_only: bool,
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    )?;

    for (name, value) in &response.headers {
        write!(stream, "{name}: {value}\r\n")?;
    }

    write!(stream, "\r\n")?;

    if !head_only {
        stream.write_all(&response.body)?;
    }

    stream.flush()
}
