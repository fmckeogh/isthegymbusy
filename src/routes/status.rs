use {
    crate::{AppState, STATUS_MAX_AGE},
    axum::{extract::State, headers::CacheControl, response::IntoResponse, Json, TypedHeader},
    serde::Serialize,
};

/// Gets current gym occupancy
pub async fn status(State(AppState { db, .. }): State<AppState>) -> impl IntoResponse {
    let capacity = sqlx::query!(
        r#"
            SELECT value FROM measurements ORDER BY measured_at DESC LIMIT 1
        "#
    )
    .fetch_one(&db)
    .await
    .unwrap()
    .value
    .try_into()
    .expect("value could not be converted from i16 to u8");

    let max_hour = sqlx::query!(
        r#"
        SELECT
            date_part('hour', measured_at)::smallint as "hour_of_day!",
            AVG(value)::smallint as "average_value!"
        FROM measurements
        GROUP BY "hour_of_day!"
        ORDER BY "average_value!" DESC
        LIMIT 1
    "#
    )
    .fetch_one(&db)
    .await
    .unwrap()
    .hour_of_day
    .try_into()
    .expect("value could not be converted from i16 to u8");

    let min_hour = sqlx::query!(
        r#"
        SELECT
            date_part('hour', measured_at)::smallint as "hour_of_day!",
            AVG(value)::smallint as "average_value!"
        FROM measurements
        GROUP BY "hour_of_day!"
        ORDER BY "average_value!" ASC
        LIMIT 1
    "#
    )
    .fetch_one(&db)
    .await
    .unwrap()
    .hour_of_day
    .try_into()
    .expect("value could not be converted from i16 to u8");

    (
        TypedHeader(
            CacheControl::new()
                .with_max_age(STATUS_MAX_AGE)
                .with_public(),
        ),
        Json(Status {
            capacity,
            max_hour,
            min_hour,
        }),
    )
}

#[derive(Debug, Serialize)]
struct Status {
    capacity: u8,
    max_hour: u8,
    min_hour: u8,
}
