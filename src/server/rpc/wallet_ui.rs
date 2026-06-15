
const HIP25_WALLET_HTML: &str = include_str!("../../../wallet/hip25/index.html");

async fn hip25_wallet_page() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        HIP25_WALLET_HTML,
    )
}