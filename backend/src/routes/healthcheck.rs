#[tracing::instrument(name = "Healthcheck endpoint")]
pub async fn healthcheck() -> impl axum::response::IntoResponse {
    axum::http::StatusCode::OK
}
