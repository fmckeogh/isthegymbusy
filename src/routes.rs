use {
    crate::{config, status::Status},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue, StatusCode,
        },
        response::IntoResponse,
    },
    maud::{html, Markup, DOCTYPE},
};

const THRESHOLD: u8 = 80;

/// Tests node health
pub async fn health() -> impl IntoResponse {
    // todo fix this
    let can_fetch_from_saint_sport = true;

    if can_fetch_from_saint_sport {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// Index page handler
pub async fn index(State(mut status): State<Status>) -> impl IntoResponse {
    let capacity = status.get().await;

    let html = html! {
        (header())

        body {
            ."p-5" {
                h2 ."text-center" { "Is the gym busy?" }

                @if capacity > THRESHOLD {
                    h1 ."display-1 text-center text-danger" { "Yes" }
                } @else {
                    h1 ."display-1 text-center text-success" { "No" }
                }

                h3 ."text-center" { "Current occupancy: " (capacity) "%" }
            }
        }
    };

    let body = minify_html::minify(html.0.as_bytes(), &minify_html::Cfg::new());

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_str(&format!(
            "public, max-age={}, immutable",
            config::get().status_validity
        ))
        .unwrap(),
    );
    (headers, body)
}

fn header() -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";

            title { "Is the gym busy?" }

            link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-GLhlTQ8iRABdZLl6O3oVMWSktQOp6b7In1Zl3/Jr59b6EGGoI1aFkw7cmDA6j6gD" crossorigin="anonymous";
        }
    }
}
