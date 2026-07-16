use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Request as HyperRequest, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::routing::any;
use tokio::sync::Semaphore;
use tower_http::timeout::TimeoutLayer;

use crate::ServerConfig;
use crate::http::{
    MAX_REQUEST_BODY_BYTES, Request, Response, first_form_values, parse_form_body_values,
    split_target,
};

const MAX_IN_FLIGHT_REQUESTS: usize = 256;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
struct AppState {
    config: ServerConfig,
    in_flight: Arc<Semaphore>,
}

/// Accept HTTP connections on `listener` and dispatch them through [`crate::route`].
pub fn serve(listener: TcpListener, config: ServerConfig) -> std::io::Result<()> {
    listener.set_nonblocking(true)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(run(listener, config))
}

async fn run(listener: TcpListener, config: ServerConfig) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)?;
    let state = AppState {
        config,
        in_flight: Arc::new(Semaphore::new(MAX_IN_FLIGHT_REQUESTS)),
    };

    let app = Router::new()
        .fallback(any(handle_request))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .with_state(state);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(std::io::Error::other)
}

async fn handle_request(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: HyperRequest,
) -> AxumResponse {
    let Ok(_permit) = state.in_flight.clone().try_acquire_owned() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Too many concurrent requests",
        )
            .into_response();
    };

    match dispatch(state.config, peer, request).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn dispatch(
    config: ServerConfig,
    peer: SocketAddr,
    request: HyperRequest,
) -> Result<AxumResponse, AxumResponse> {
    let method = request.method().as_str().to_string();
    let head_only = method.eq_ignore_ascii_case("HEAD");

    let target = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let (path, query) = split_target(&target);

    let mut headers = HashMap::new();
    for (name, value) in request.headers() {
        if let Ok(value) = value.to_str() {
            headers.insert(name.as_str().to_ascii_lowercase(), value.to_string());
        }
    }
    headers
        .entry("x-robominer-peer".to_string())
        .or_insert_with(|| peer.ip().to_string());

    let body = axum::body::to_bytes(request.into_body(), MAX_REQUEST_BODY_BYTES)
        .await
        .map_err(|_| to_axum_response(&Response::payload_too_large(), false))?
        .to_vec();

    let form_values = if method.eq_ignore_ascii_case("POST") {
        parse_form_body_values(&headers, &body)
    } else {
        HashMap::new()
    };
    let form = first_form_values(&form_values);

    let app_request = Request {
        method,
        path,
        query,
        form,
        form_values,
        headers,
    };
    let response = crate::route(&app_request, &config).await;
    Ok(to_axum_response(&response, head_only))
}

fn to_axum_response(response: &Response, head_only: bool) -> AxumResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = AxumResponse::builder()
        .status(status)
        .header(header::CONTENT_TYPE, response.content_type)
        .header(header::CONTENT_LENGTH, response.body.len());

    for (name, value) in &response.headers {
        if let Ok(header_value) = HeaderValue::from_str(value) {
            builder = builder.header(*name, header_value);
        }
    }

    let body = if head_only {
        Body::empty()
    } else {
        Body::from(response.body.clone())
    };

    builder.body(body).unwrap_or_else(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    })
}
