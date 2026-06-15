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

/// Max blocks per block/datas request.
pub const BLOCK_DATAS_MAX_LIMIT: u64 = 500;

const SECRET_QUERY_MARKERS: &[&str] = &[
    "prikey=",
    "main_prikey=",
    "from_prikey=",
    "fee_prikey=",
    "password=",
];

const RATE_LIMIT_MAX_KEYS: usize = 4096;

pub fn is_mainnet(chain_id: u64) -> bool {
    chain_id != HIP25_DEV_CHAIN_ID
}

pub fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

pub fn is_public_bind_host(host: &str) -> bool {
    host == "0.0.0.0" || host == "::" || host == "[::]"
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

pub fn query_string_has_secret_keys(query: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    let q = query.to_ascii_lowercase();
    SECRET_QUERY_MARKERS.iter().any(|m| q.contains(m))
}

pub fn path_accepts_signing_secrets(path: &str) -> bool {
    path == "/util/transaction/sign"
        || path == "/create/coin/transfer"
        || path == "/operate/fee/raise"
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

    fn prune_stale(map: &mut HashMap<String, (u32, Instant)>, now: Instant, window: Duration) {
        if map.len() <= RATE_LIMIT_MAX_KEYS {
            return;
        }
        map.retain(|_, (_, start)| now.duration_since(*start) <= window);
        if map.len() > RATE_LIMIT_MAX_KEYS {
            map.clear();
        }
    }

    pub fn allow(&self, key: &str) -> bool {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();
        Self::prune_stale(&mut map, now, self.window);
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
        || path.starts_with("/create/")
        || path.starts_with("/util/")
}

fn is_rate_limited_post(path: &str, method: &Method) -> bool {
    *method == Method::POST
        && (path.starts_with("/submit/")
            || path.starts_with("/operate/")
            || path.starts_with("/create/")
            || path.starts_with("/util/"))
}

fn is_rate_limited_get(path: &str, method: &Method) -> bool {
    *method == Method::GET && path.starts_with("/submit/miner/")
}

pub fn origin_allowed(origin: &str, listen_host: &str, listen_port: u16) -> bool {
    let o = origin.trim();
    if is_loopback_host(listen_host) {
        if o.is_empty() {
            return true;
        }
        return o.starts_with(&format!("http://127.0.0.1:{listen_port}"))
            || o.starts_with(&format!("http://localhost:{listen_port}"))
            || o.starts_with(&format!("https://127.0.0.1:{listen_port}"))
            || o.starts_with(&format!("https://localhost:{listen_port}"));
    }
    if o.is_empty() {
        return false;
    }
    let host = listen_host.trim();
    o.starts_with(&format!("http://{host}:{listen_port}"))
        || o.starts_with(&format!("https://{host}:{listen_port}"))
        || o == format!("http://{host}")
        || o == format!("https://{host}")
}

#[derive(Clone)]
pub struct MiddlewareCtx {
    pub listen_host: String,
    pub listen_port: u16,
    pub rate_limiter: Arc<RateLimiter>,
}

pub async fn security_middleware(
    axum::Extension(mw): axum::Extension<MiddlewareCtx>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let query = request.uri().query().unwrap_or("").to_string();

    if query_string_has_secret_keys(&query)
        && (path_accepts_signing_secrets(&path) || path.starts_with("/util/"))
    {
        return (
            StatusCode::BAD_REQUEST,
            api_error(
                "secrets must be sent in POST JSON body, never in URL query strings",
            ),
        )
            .into_response();
    }

    if is_mutating_path(&path) {
        let origin_hdr = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !origin_allowed(origin_hdr, &mw.listen_host, mw.listen_port) {
            return (
                StatusCode::FORBIDDEN,
                api_error("cross-origin mutation blocked; use the local wallet UI"),
            )
                .into_response();
        }
    }

    if is_rate_limited_post(&path, &method) || is_rate_limited_get(&path, &method) {
        let ip = client_ip(&request);
        let verb = if method == Method::POST { "post" } else { "get" };
        let rate_key = format!("{verb}:{ip}:{}", path);
        if !mw.rate_limiter.allow(&rate_key) {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                api_error("rate limit exceeded, retry later"),
            )
                .into_response();
        }
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

pub fn resolve_listen_endpoint(host: &str, port: u16, allow_public_rpc: bool) -> Result<String, String> {
    let h = host.trim();
    if is_public_bind_host(h) && !allow_public_rpc {
        return Err(format!(
            "listen_host={h} requires allow_public_rpc=true in [server] (default is loopback-only)"
        ));
    }
    Ok(h.to_string())
}