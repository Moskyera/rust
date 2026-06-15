use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::{ConnectInfo, Request},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::config::HIP25_DEV_CHAIN_ID;

use super::ctx::api_error;

/// Max RPC POST body (signed tx, hex payloads).
pub const RPC_BODY_LIMIT_BYTES: usize = 256 * 1024;

/// Max diamonds processed per staking/summary request.
pub const STAKING_SUMMARY_MAX_DIAMONDS: usize = 200;

pub fn is_mainnet(chain_id: u64) -> bool {
    chain_id != HIP25_DEV_CHAIN_ID
}

pub fn server_signing_disabled_msg() -> &'static str {
    "server-side secret signing is disabled on mainnet; use client-side WASM signing"
}

pub fn reject_server_secret_signing(chain_id: u64) -> Option<&'static str> {
    if is_mainnet(chain_id) {
        Some(server_signing_disabled_msg())
    } else {
        None
    }
}

pub fn reject_create_account_on_mainnet(chain_id: u64) -> Option<&'static str> {
    if is_mainnet(chain_id) {
        Some("create/account is disabled on mainnet RPC")
    } else {
        None
    }
}

/// Sliding-window per-IP rate limiter for expensive RPC routes.
pub struct RateLimiter {
    inner: Mutex<HashMap<String, (u32, Instant)>>,
    max_per_window: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_per_window: u32, window_secs: u64) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max_per_window,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn allow(&self, key: &str) -> bool {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > self.window {
            *entry = (0, now);
        }
        if entry.0 >= self.max_per_window {
            return false;
        }
        entry.0 += 1;
        true
    }
}

fn client_ip(request: &Request) -> String {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn is_mutating_path(path: &str) -> bool {
    path.starts_with("/submit/")
        || path.starts_with("/operate/")
        || path == "/util/transaction/sign"
        || path == "/create/coin/transfer"
        || path == "/create/transaction"
}

fn origin_allowed(origin: &str, listen_host: &str) -> bool {
    let o = origin.trim();
    if o.is_empty() {
        return true;
    }
    if o.starts_with("http://127.0.0.1:")
        || o.starts_with("http://localhost:")
        || o.starts_with("https://127.0.0.1:")
        || o.starts_with("https://localhost:")
    {
        return true;
    }
    if listen_host == "127.0.0.1" || listen_host == "localhost" {
        return false;
    }
    format!("http://{listen_host}").starts_with(o)
        || format!("https://{listen_host}").starts_with(o)
}

#[derive(Clone)]
pub struct MiddlewareCtx {
    pub listen_host: String,
    pub rate_limiter: Arc<RateLimiter>,
}

pub async fn security_middleware(
    axum::Extension(mw): axum::Extension<MiddlewareCtx>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    if is_mutating_path(&path) {
        if let Some(origin) = request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
            if !origin_allowed(origin, &mw.listen_host) {
                return (
                    StatusCode::FORBIDDEN,
                    api_error("cross-origin mutation blocked; use the local wallet UI"),
                )
                    .into_response();
            }
        }
    }

    let ip = client_ip(&request);
    let rate_key = if path.starts_with("/submit/transaction") {
        format!("submit:{ip}")
    } else if path.starts_with("/util/transaction/") {
        format!("util:{ip}")
    } else {
        String::new()
    };

    if !rate_key.is_empty() && !mw.rate_limiter.allow(&rate_key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            api_error("rate limit exceeded, retry later"),
        )
            .into_response();
    }

    if method == Method::GET && path == "/create/coin/transfer" {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            api_error(
                "create/coin/transfer requires POST; never put prikey/password in URL query strings",
            ),
        )
            .into_response();
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if path.starts_with("/hip25/wallet") {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'none'; object-src 'none'; base-uri 'self'",
            ),
        );
    }
    if path.starts_with("/pkg/") {
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
    }
    response
}