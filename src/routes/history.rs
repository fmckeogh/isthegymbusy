use {
    crate::{AppState, HISTORY_MAX_AGE},
    axum::{
        extract::State,
        http::{
            header::{CACHE_CONTROL, CONTENT_TYPE},
            HeaderMap, HeaderValue,
        },
        response::IntoResponse,
    },
    chrono::{DateTime, Utc},
};

/// Number of seconds in a 5 minute interval
const INTERVAL: u64 = 60 * 5;

/// Number of seconds in a day
const DAY_SECONDS: u64 = 24 * 60 * 60;

/// Number of intervals in two days
const NUM_INTERVALS: u64 = (2 * DAY_SECONDS) / INTERVAL;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub value: u8,
    pub timestamp: i64,
}

pub async fn history(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    let start_timestamp = Utc::now().timestamp();

    let mut body = Vec::with_capacity(NUM_INTERVALS as usize);

    let history = {
        struct DbEntry {
            measured_at: DateTime<Utc>,
            value: i16,
        }

        sqlx::query_as!(
            DbEntry,
            "SELECT * FROM measurements
            WHERE measured_at >= NOW() - INTERVAL '48 HOURS'
            ORDER BY measured_at DESC"
        )
        .fetch_all(&db)
        .await
        .unwrap()
        .into_iter()
        .map(|DbEntry { measured_at, value }| Entry {
            timestamp: measured_at.timestamp(),
            value: value.try_into().unwrap(),
        })
        .collect::<Vec<_>>()
    };

    let mut history_iter = history.iter().peekable();

    //
    for bucket_idx in 1..=NUM_INTERVALS {
        let end = start_timestamp - i64::try_from(bucket_idx * INTERVAL).unwrap();

        let mut bucket = vec![];

        loop {
            if history_iter
                .peek()
                .unwrap_or(&&Entry {
                    timestamp: 0,
                    value: 0,
                })
                .timestamp
                > end
            {
                bucket.push(history_iter.next().unwrap().value as u16);
            } else {
                break;
            }
        }

        let mean = if bucket.len() == 0 {
            0xFF
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
