use axum::{http::Uri, response::IntoResponse};

mod health;
mod history;
mod static_files;
mod status;

pub use {health::health, history::history, static_files::static_files, status::status};

pub async fn index() -> impl IntoResponse {
    static_files(Uri::from_static("/index.html")).await
}
