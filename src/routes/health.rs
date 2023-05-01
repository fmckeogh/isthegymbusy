use {
    crate::AppState,
    axum::{extract::State, http::StatusCode, response::IntoResponse},
};

/// Tests node health
pub async fn health(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    if sqlx::query("SELECT 1").fetch_one(&db).await.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
