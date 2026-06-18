use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::state::GatewayState;

pub async fn login_relay_handler(
    State(app_state): State<Arc<GatewayState>>,
    request: Request,
) -> Response {
    relay_auth_request(&app_state, request, "/auth/login").await
}

pub async fn register_relay_handler(
    State(app_state): State<Arc<GatewayState>>,
    request: Request,
) -> Response {
    relay_auth_request(&app_state, request, "/auth/register").await
}

async fn relay_auth_request(
    app_state: &Arc<GatewayState>,
    request: Request,
    upstream_path: &str,
) -> Response {
    let (parts, body) = request.into_parts();
    let body = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let upstream_url = format!(
        "{}{}",
        app_state
            .gateway
            .upstream_api_base_url
            .trim_end_matches('/'),
        upstream_path
    );
    let mut upstream_request = app_state.http_client.post(upstream_url).body(body.to_vec());

    if let Some(content_type) = parts.headers.get(header::CONTENT_TYPE) {
        upstream_request = upstream_request.header(header::CONTENT_TYPE, content_type.clone());
    }

    if let Some(real_ip) = parts.headers.get("x-real-ip") {
        upstream_request = upstream_request.header("x-real-ip", real_ip.clone());
    }

    let upstream_response = match upstream_request.send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let status = upstream_response.status();
    let upstream_headers = upstream_response.headers().clone();
    let body = match upstream_response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let mut response_headers = HeaderMap::new();
    for (name, value) in &upstream_headers {
        if name == header::TRANSFER_ENCODING || name == header::SET_COOKIE {
            continue;
        }
        response_headers.append(name, value.clone());
    }

    if !response_headers.contains_key(header::CONTENT_TYPE) {
        response_headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }

    (status, response_headers, Body::from(body)).into_response()
}
