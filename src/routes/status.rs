use {
    crate::{config, AppState, STATUS_MAX_AGE_DIVISOR},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
    std::sync::atomic::Ordering::Relaxed,
};

/// Gets current gym occupancy
pub async fn status(State(AppState { capacity, .. }): State<AppState>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_str(&format!(
            "public, max-age={}, immutable",
            config::get().fetch_interval / STATUS_MAX_AGE_DIVISOR
        ))
        .unwrap(),
    );

    (headers, [capacity.load(Relaxed)])
}
