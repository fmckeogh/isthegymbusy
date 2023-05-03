use {
    crate::STATIC_FILES_MAX_AGE,
    axum::{
        headers::{CacheControl, ContentType},
        http::{StatusCode, Uri},
        response::{IntoResponse, Response},
        TypedHeader,
    },
    include_dir::{include_dir, Dir},
};

static DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// Handler for static files
pub async fn static_files(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    let Some(file) = DIR.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    (
        TypedHeader(ContentType::from(mime_type)),
        TypedHeader(
            CacheControl::new()
                .with_max_age(STATIC_FILES_MAX_AGE)
                .with_public(),
        ),
        file.contents(),
    )
        .into_response()
}
