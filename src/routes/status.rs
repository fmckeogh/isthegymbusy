use {
    crate::{AppState, STATUS_MAX_AGE_DIVISOR},
    axum::{extract::State, response::IntoResponse},
    axum_extra::{
        headers::{CacheControl, ContentType},
        TypedHeader,
    },
    mime_guess::mime::APPLICATION_OCTET_STREAM,
    std::{sync::atomic::Ordering::Relaxed, time::Duration},
};

/// Gets current gym occupancy
pub async fn status(
    State(AppState {
        capacity, config, ..
    }): State<AppState>,
) -> impl IntoResponse {
    (
        TypedHeader(ContentType::from(APPLICATION_OCTET_STREAM)),
        TypedHeader(
            CacheControl::new()
                .with_max_age(Duration::from_secs(
                    config.fetch_interval / STATUS_MAX_AGE_DIVISOR,
                ))
                .with_public(),
        ),
        [capacity.load(Relaxed)],
    )
}
