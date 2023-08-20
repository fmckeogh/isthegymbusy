//! Gets the busyness history for the current day

use {
    crate::{AppState, HISTORY_MAX_AGE},
    axum::{
        extract::State,
        headers::{self, CacheControl, ContentType, Header},
        http::{HeaderName, HeaderValue},
        response::IntoResponse,
        TypedHeader,
    },
    chrono::{DateTime, Utc},
    mime_guess::mime::APPLICATION_OCTET_STREAM,
    sqlx::postgres::types::PgInterval,
    std::{iter::once, time::Duration},
};

/// Size of time intervals in which to group and average measurements in
const INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub value: u8,
    pub timestamp: i64,
}

pub async fn today(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    struct DbEntry {
        measured_at: DateTime<Utc>,
        value: i16,
    }

    // get entries today from 6:00 to 22:00
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
                        date_trunc('day', NOW()) + interval '6 hours',
                        date_trunc('day', NOW()) + interval '22 hours',
                        $1::interval
                    ) as int_start
            ) as intervals
            LEFT JOIN measurements ON (
                measurements.measured_at >= intervals.int_start AND
                measurements.measured_at < intervals.int_start + $1::interval
            )
            GROUP BY intervals.int_start
            ORDER BY intervals.int_start DESC;
        "#,
        PgInterval::try_from(INTERVAL).unwrap()
    )
    .fetch_all(&db)
    .await
    .unwrap();

    let latest_timestamp = history.first().unwrap().measured_at;

    let body = history
        .into_iter()
        .map(|DbEntry { value, .. }| value.try_into().unwrap())
        .collect::<Vec<u8>>();

    (
        TypedHeader(ContentType::from(APPLICATION_OCTET_STREAM)),
        TypedHeader(
            CacheControl::new()
                .with_max_age(HISTORY_MAX_AGE)
                .with_public(),
        ),
        TypedHeader(HistoryLatest(latest_timestamp)),
        TypedHeader(HistoryInterval(INTERVAL)),
        body,
    )
}

struct HistoryLatest(DateTime<Utc>);

impl Header for HistoryLatest {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("history-latest");
        &NAME
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0.timestamp().to_string()).unwrap();
        values.extend(once(value));
    }
}

struct HistoryInterval(Duration);

impl Header for HistoryInterval {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("history-interval");
        &NAME
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0.as_secs().to_string()).unwrap();
        values.extend(std::iter::once(value));
    }
}
