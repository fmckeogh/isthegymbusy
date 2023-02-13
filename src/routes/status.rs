use {
    crate::{config, StatusFetcher, STATUS_MAX_AGE_DIVISOR},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
};

/// Gets current gym occupancy
pub async fn status(State(status): State<StatusFetcher>) -> impl IntoResponse {
    let capacity = status.capacity().await;

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

    (headers, [capacity])
}
