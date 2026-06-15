
const HIP25_WALLET_HTML: &str = include_str!("../../../wallet/hip25/index.html");

fn hip25_pkg_dir() -> Option<std::path::PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let candidates = [
        exe_dir.join("pkg"),
        exe_dir.join("wallet").join("hip25").join("pkg"),
    ];
    for p in candidates {
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

fn serve_pkg_file(name: &str, content_type: &'static str) -> Response {
    use axum::http::StatusCode;
    let Some(dir) = hip25_pkg_dir() else {
        return (
            StatusCode::NOT_FOUND,
            "HIP-25 WASM SDK not built; run scripts/build_wallet_sdk.ps1",
        )
            .into_response();
    };
    let path = dir.join(name);
    match std::fs::read(&path) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, content_type),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, format!("missing {}", name)).into_response(),
    }
}

async fn hip25_wallet_page() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        HIP25_WALLET_HTML,
    )
}

async fn hip25_sdk_js() -> impl IntoResponse {
    serve_pkg_file("hacash_sdk.js", "application/javascript")
}

async fn hip25_sdk_wasm() -> impl IntoResponse {
    serve_pkg_file("hacash_sdk_bg.wasm", "application/wasm")
}