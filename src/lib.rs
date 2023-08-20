use {
    crate::{
        log::{create_trace_layer, tracing_init},
        routes::{health, history, index, static_files, status},
        status::StatusFetcher,
    },
    axum::{routing::get, Router},
    color_eyre::eyre::Result,
    sqlx::{
        postgres::{PgPoolOptions, Postgres},
        Pool,
    },
    std::{
        net::SocketAddr,
        sync::{atomic::AtomicU8, Arc},
        time::Duration,
    },
    tokio::task::JoinHandle,
    tower_http::compression::CompressionLayer,
    tracing::{debug, info},
};

pub mod config;
pub mod error;
pub mod log;
pub mod routes;

pub mod status;

pub use crate::config::Config;

/// status.bin cached for half of the fetch interval
const STATUS_MAX_AGE_DIVISOR: u64 = 2;

/// history.bin cached for 15 minutes
const HISTORY_MAX_AGE: Duration = Duration::from_secs(15 * 60);

/// Static files cached for 15 minutes
const STATIC_FILES_MAX_AGE: Duration = Duration::from_secs(15 * 60);

const DATABASE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
const DATABASE_MIN_CONNECTIONS: u32 = 5;

#[derive(Clone)]
pub struct AppState {
    capacity: Arc<AtomicU8>,
    db: Pool<Postgres>,
    config: Config,
}

/// Starts a new instance, returning a handle
pub async fn start(config: &Config) -> Result<Handle> {
    // initialize global tracing subscriber
    tracing_init()?;

    let _guard = sentry::init((
        config.sentry_url.as_str(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let db = PgPoolOptions::new()
        .acquire_timeout(DATABASE_ACQUIRE_TIMEOUT)
        .min_connections(DATABASE_MIN_CONNECTIONS)
        .connect(&config.database_url)
        .await
        .unwrap();

    debug!("running migrations");
    sqlx::migrate!().run(&db).await?;

    let capacity =
        StatusFetcher::init(db.clone(), Duration::from_secs(config.fetch_interval)).await;

    let compression = CompressionLayer::new().br(true).deflate(true).gzip(true);

    // create router with all routes and tracing layer
    let router = Router::new()
        .route("/health", get(health))
        .route("/", get(index))
        .route("/history/today", get(history::today))
        .route("/history/average", get(history::average))
        .route("/history/year", get(history::year))
        .route("/status", get(status))
        .fallback(static_files)
        .with_state(AppState {
            capacity,
            db,
            config: config.clone(),
        })
        .layer(compression)
        .layer(create_trace_layer());

    // bind axum server to socket address and use router to create a service factory
    let server = axum::Server::bind(&config.address).serve(router.into_make_service());

    // get address server is bound to (may be different to address passed to Server::bind)
    let address = server.local_addr();

    // spawn server on new tokio task
    let handle = tokio::spawn(async { server.await.map_err(Into::into) });

    info!("isthegymbusy started on http://{}", address);

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

    _guard: sentry::ClientInitGuard,
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
