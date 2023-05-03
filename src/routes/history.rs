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
    sqlx::postgres::types::PgInterval,
    std::time::Duration,
};

/// Window in which to retrieve measurements from
const QUERY_WINDOW: Duration = Duration::from_secs(60 * 60 * 24 * 2);

/// Size of time intervals in which to group and average measurements in
const INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub value: u8,
    pub timestamp: i64,
}

pub async fn history(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    struct DbEntry {
        measured_at: DateTime<Utc>,
        value: i16,
    }

    let history = sqlx::query_as!(
        DbEntry,
        r#"
            SELECT
                intervals.int_start as "measured_at!",
                CASE
                    WHEN COUNT(measurements.value) > 0 THEN AVG(measurements.value)::smallint
                    ELSE 255::smallint
                END as "value!"
            FROM (
                SELECT
                    generate_series(
                        date_trunc('minute', NOW() - $1::interval),
                        NOW(),
                        $2::interval
                    ) as int_start
            ) as intervals
            LEFT JOIN measurements ON (
                measurements.measured_at >= intervals.int_start AND
                measurements.measured_at < intervals.int_start + $2::interval
            )
            GROUP BY intervals.int_start
            ORDER BY intervals.int_start DESC;
        "#,
        PgInterval::try_from(QUERY_WINDOW).unwrap(),
        PgInterval::try_from(INTERVAL).unwrap()
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let latest_timestamp = history.first().unwrap().measured_at.timestamp().to_string();

    let body = history
        .into_iter()
        .map(|DbEntry { value, .. }| value.try_into().unwrap())
        .collect::<Vec<u8>>();

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
        "history-latest",
        HeaderValue::from_str(&latest_timestamp).unwrap(),
    );
    headers.insert(
        "history-interval",
        HeaderValue::from_str(&INTERVAL.as_secs().to_string()).unwrap(),
    );

    (headers, body)
}
