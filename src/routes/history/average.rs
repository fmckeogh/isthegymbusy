//! Gets the historical average busyness for this day

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
    std::{iter::once, time::Duration},
};

/// Size of time intervals in which to group and average measurements in
const INTERVAL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub value: u8,
    pub timestamp: i64,
}

pub async fn average(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    struct DbEntry {
        measured_at: DateTime<Utc>,
        value: i16,
    }

    // get average for entries in the past few days
    let history = sqlx::query_as!(
        DbEntry,
        r#"
        SELECT
            date_trunc('day', NOW()) + interval '15 minutes' * intervals.int_start  as "measured_at!",
            CASE
                WHEN COUNT(measurements.value) > 0 THEN AVG(measurements.value)::smallint
                ELSE 255::smallint
            END as "value!"
        FROM (
            SELECT
                generate_series(
                    6 * 4,
                    22 * 4
                ) as int_start
        ) as intervals
        LEFT JOIN measurements ON (
            measurements.measured_at > NOW() - interval '7 days' AND
            measurements.measured_at >= date_trunc('day', measurements.measured_at) + (interval '15 minutes' * intervals.int_start) AND
            measurements.measured_at < date_trunc('day', measurements.measured_at) + (interval '15 minutes' * intervals.int_start) + interval '15 minutes' AND
            measurements.value > 0
        )
        GROUP BY intervals.int_start
        ORDER BY intervals.int_start DESC
        "#,
        // PgInterval::try_from(INTERVAL).unwrap()
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
