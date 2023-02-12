use {
    crate::{history::Entry, status::StatusFetcher},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
    std::fmt::Write,
};

pub async fn history(State(status): State<StatusFetcher>) -> impl IntoResponse {
    let mut body = String::new();

    status
        .history()
        .await
        .iter()
        .for_each(|Entry { timestamp, value }| writeln!(body, "{timestamp} {value}").unwrap());

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600, immutable"),
    );

    (headers, body)
}
