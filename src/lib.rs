use {
    crate::{
        log::create_trace_layer,
        routes::{health, history, index, static_files, status},
        status::StatusFetcher,
    },
    axum::{routing::get, Router},
    color_eyre::eyre::Result,
    std::net::SocketAddr,
    tokio::task::JoinHandle,
    tower_http::compression::CompressionLayer,
    tracing::info,
    tracing_appender::non_blocking::WorkerGuard,
};

pub mod config;
pub mod error;
pub mod history;
pub mod log;
pub mod routes;

pub mod status;

pub use crate::config::Config;

/// status.bin cached for half of the fetch interval
const STATUS_MAX_AGE_DIVISOR: u64 = 2;

/// history.bin cached for 15 minutes
const HISTORY_MAX_AGE: u64 = 15 * 60;

/// Static files cached for 15 minutes
const STATIC_FILES_MAX_AGE: u64 = 15 * 60;

/// Starts a new instance of the contractor returning a handle
pub async fn start(config: &Config) -> Result<Handle> {
    // initialize global tracing subscriber
    let _guard = crate::log::tracing_init(&config.log_path)?;

    config::init(config.clone()).await;

    let status_fetcher = StatusFetcher::new().await;

    let compression = CompressionLayer::new().br(true).deflate(true).gzip(true);

    // create router with all routes and tracing layer
    let router = Router::new()
        .route("/health", get(health))
        .route("/", get(index))
        .route("/history.bin", get(history))
        .route("/status.bin", get(status))
        .fallback(static_files)
        .with_state(status_fetcher)
        .layer(compression)
        .layer(create_trace_layer());

    // bind axum server to socket address and use router to create a service factory
    let server = axum::Server::bind(&config::get().address).serve(router.into_make_service());

    // get address server is bound to (may be different to address passed to Server::bind)
    let address = server.local_addr();

    // spawn server on new tokio task
    let handle = tokio::spawn(async { server.await.map_err(Into::into) });

    info!("contractor started on http://{}", address);

    // return handles
    Ok(Handle {
        address,
        handle,
        _guard,
    })
}

/// Handle for running an instance
pub struct Handle {
    // Socket address instance is bound to
    address: SocketAddr,
    // JoinHandle for server task
    handle: JoinHandle<Result<()>>,
    // Guard for log file tracing
    _guard: WorkerGuard,
}

impl Handle {
    /// Gets the socket address the running instance is bound to
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Awaits on the instance's task
    pub async fn join(self) -> Result<()> {
        self.handle.await??;
        Ok(())
    }
}
