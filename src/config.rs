//! App configuration

use {
    color_eyre::eyre::{Result, WrapErr},
    config::Environment,
    serde::Deserialize,
    std::net::SocketAddr,
};

/// Contractor configuration parameters
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// Socket to bind HTTP server to
    pub address: SocketAddr,

    /// Number of seconds between fetching status
    pub fetch_interval: u64,

    /// Postgres URL
    pub database_url: String,
}

impl Config {
    /// Builds a new Config instance from an optional file (the path of which is supplied as a argument) and, with a greater priority, environment variables
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(config::Config::builder()
            .add_source(Environment::default())
            .build()
            .wrap_err("Failed build configuration")?
            .try_deserialize()
            .wrap_err("Failed deserialize configuration")?)
    }
}
