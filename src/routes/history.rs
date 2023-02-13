use {
    crate::{history::Entry, status::StatusFetcher, HISTORY_MAX_AGE},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
    chrono::Utc,
};

/// Number of seconds in a 5 minute interval
const INTERVAL: u64 = 60 * 5;

/// Number of seconds in a day
const DAY_SECONDS: u64 = 24 * 60 * 60;

/// Number of intervals in two days
const NUM_INTERVALS: u64 = (2 * DAY_SECONDS) / INTERVAL;

pub async fn history(State(status): State<StatusFetcher>) -> impl IntoResponse {
    let start_timestamp = Utc::now().timestamp();

    let mut body = Vec::with_capacity(NUM_INTERVALS as usize);

    let history = status.history().await;
    let mut history_rev = history.iter().rev().peekable();

    //
    for bucket_idx in 1..=NUM_INTERVALS {
        let end = start_timestamp - i64::try_from(bucket_idx * INTERVAL).unwrap();

        let mut bucket = vec![];

        loop {
            if history_rev
                .peek()
                .unwrap_or(&&Entry {
                    timestamp: 0,
                    value: 0,
                })
                .timestamp
                > end
            {
                bucket.push(history_rev.next().unwrap().value as u16);
            } else {
                break;
            }
        }

        let mean = if bucket.len() == 0 {
            0
        } else {
            u8::try_from(bucket.iter().sum::<u16>() / u16::try_from(bucket.len()).unwrap()).unwrap()
        };

        body.push(mean)
    }

    assert!(body.len() == NUM_INTERVALS as usize);

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_str(&format!("public, max-age={HISTORY_MAX_AGE}, immutable")).unwrap(),
    );

    headers.insert(
        "history-end",
        HeaderValue::from_str(&start_timestamp.to_string()).unwrap(),
    );
    headers.insert(
        "history-interval",
        HeaderValue::from_str(&INTERVAL.to_string()).unwrap(),
    );

    (headers, body)
}
