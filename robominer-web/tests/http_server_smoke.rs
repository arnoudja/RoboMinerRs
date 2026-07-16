use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use robominer_web::{ServerConfig, serve};

/// Must match `robominer_web::http::MAX_REQUEST_BODY_BYTES`.
const MAX_REQUEST_BODY_BYTES: usize = 1_048_576;

fn static_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}

fn spawn_server() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind smoke listener");
    let addr = listener.local_addr().expect("local addr");
    let handle = thread::spawn(move || {
        serve(
            listener,
            ServerConfig {
                static_root: static_root(),
                database_pool: None,
                allow_signup: false,
                trust_proxy: false,
            },
        )
        .expect("serve");
    });
    // Give the runtime a moment to accept.
    thread::sleep(Duration::from_millis(50));
    (format!("{addr}"), handle)
}

fn raw_http_exchange(addr: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout");
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("write timeout");
    stream.write_all(request.as_bytes()).expect("write request");
    stream.flush().expect("flush");

    let mut response = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buffer[..n]),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(error) => panic!("read failed: {error}"),
        }
    }
    String::from_utf8_lossy(&response).into_owned()
}

#[test]
fn serve_returns_static_css_and_rejects_oversized_body() {
    let (addr, _handle) = spawn_server();

    let get_response = raw_http_exchange(
        &addr,
        "GET /css/robominer.css HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(
        get_response.starts_with("HTTP/1.1 200"),
        "expected 200 for static CSS, got:\n{get_response}"
    );
    let get_lower = get_response.to_ascii_lowercase();
    assert!(
        get_lower.contains("content-type: text/css"),
        "expected text/css content type, got:\n{get_response}"
    );
    assert!(
        get_lower.contains("cache-control: public, max-age=604800"),
        "expected Cache-Control header, got:\n{get_response}"
    );
    assert!(
        get_lower.contains("etag:"),
        "expected ETag header, got:\n{get_response}"
    );
    assert!(
        get_lower.contains("x-content-type-options: nosniff"),
        "expected X-Content-Type-Options, got:\n{get_response}"
    );
    assert!(
        get_lower.contains("x-frame-options: sameorigin"),
        "expected X-Frame-Options, got:\n{get_response}"
    );
    assert!(
        get_lower.contains("referrer-policy: strict-origin-when-cross-origin"),
        "expected Referrer-Policy, got:\n{get_response}"
    );

    let oversized = MAX_REQUEST_BODY_BYTES + 1;
    let post_response = raw_http_exchange(
        &addr,
        &format!(
            "POST /login HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {oversized}\r\nConnection: close\r\n\r\n{}",
            "x".repeat(oversized)
        ),
    );
    assert!(
        post_response.starts_with("HTTP/1.1 413"),
        "expected 413 for oversized body, got:\n{post_response}"
    );
}
