use {
    crate::status::StatusFetcher,
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
};

pub async fn history(State(status): State<StatusFetcher>) -> impl IntoResponse {
    let body = status
        .history()
        .await
        .iter()
        .map(|e| e.value)
        .collect::<Vec<_>>();

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400, immutable"),
    );

    (headers, body)
}
