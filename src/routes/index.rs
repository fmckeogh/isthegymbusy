use {
    crate::{config, status::StatusFetcher},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
    maud::{html, DOCTYPE},
};

const THRESHOLD: u8 = 80;

/// Index page handler
pub async fn index(State(status): State<StatusFetcher>) -> impl IntoResponse {
    let capacity = status.capacity().await;

    let html = html! {
        (DOCTYPE)

        head {
            meta charset="utf-8";

            title { "Is the gym busy?" }

            link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/bootstrap/5.3.0-alpha1/css/bootstrap.min.css";
            script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.2.1/chart.umd.min.js" {}
            script src="https://cdnjs.cloudflare.com/ajax/libs/luxon/3.2.1/luxon.min.js" {}
            script src="https://cdnjs.cloudflare.com/ajax/libs/chartjs-adapter-luxon/1.3.1/chartjs-adapter-luxon.umd.min.js" {}

            link rel="stylesheet" href="/style.css";
            script src="/main.js" type="module" {}
        }

        body {
            ."p-5" {
                h2 ."text-center" style="font-size: 400%;" { "Is the gym busy?" }

                @if capacity > THRESHOLD {
                    h1 ."display-1 text-center text-danger" style="font-size: 1500%;" { "Yes" }
                } @else {
                    h1 ."display-1 text-center text-success" style="font-size: 1500%;" { "No" }
                }

                h3 ."text-center" style="font-size: 500%;" { "Current occupancy: " (capacity) "%" }

                ."py-5" {
                    h4 ."text-center" style="font-size: 300%;" { "48-hour History" }
                    canvas #"chart" {}
                }
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
            config::get().fetch_interval / 4
        ))
        .unwrap(),
    );

    (headers, body)
}
