use {
    crate::{
        config,
        history::{Entry, PersistentHistory},
    },
    futures::lock::Mutex,
    regex::Regex,
    reqwest::{Client, ClientBuilder},
    std::{
        num::ParseIntError,
        sync::Arc,
        time::{Duration, Instant},
    },
    tokio::time::interval,
    tracing::{error, info},
};

const URL: &'static str = "https://sport.wp.st-andrews.ac.uk/";

#[derive(Clone)]
pub struct StatusFetcher(Arc<Mutex<Inner>>);

impl StatusFetcher {
    pub async fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let inner = Arc::new(Mutex::new(Inner {
            capacity: 0,
            last_fetch: Instant::now(),
            client,
            regex: Regex::new(r"Occupancy: ([0-9]+)%").unwrap(),
            history: PersistentHistory::open(&config::get().history_path),
        }));

        tokio::spawn(fetcher_task(inner.clone()));

        Self(inner)
    }

    pub async fn capacity(&self) -> u8 {
        self.0.lock().await.capacity
    }

    pub async fn history(&self) -> Vec<Entry> {
        self.0.lock().await.history.get()
    }
}

async fn fetcher_task(inner: Arc<Mutex<Inner>>) {
    let mut interval = interval(Duration::from_secs(config::get().fetch_interval));
    loop {
        interval.tick().await;
        if let Err(e) = inner.lock().await.update_status().await {
            error!("Error while updating status: {e:?}");
        }
    }
}

pub struct Inner {
    capacity: u8,
    last_fetch: Instant,
    client: Client,
    regex: Regex,
    history: PersistentHistory,
}

/// Error occurred while updating status
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum StatusUpdateError {
    /// Error during GET request
    Request(#[from] reqwest::Error),
    /// Regex did not match response text {text}
    MissingCaptures { text: String },
    /// No capture group found at index {i} in text {text}
    MissingCaptureGroup { text: String, i: usize },
    /// Failed to parse {1:?} as u8: {0:?}
    Parse(ParseIntError, String),
}

impl Inner {
    async fn update_status(&mut self) -> Result<(), StatusUpdateError> {
        info!("Starting status fetch");

        let text = self.client.get(URL).send().await?.text().await?;

        let captures = self
            .regex
            .captures(&text)
            .ok_or_else(|| StatusUpdateError::MissingCaptures { text: text.clone() })?;

        let percentage = captures
            .get(1)
            .ok_or_else(|| StatusUpdateError::MissingCaptureGroup {
                text: text.clone(),
                i: 1,
            })?
            .as_str();

        self.capacity = percentage
            .parse()
            .map_err(|e| StatusUpdateError::Parse(e, percentage.to_owned()))?;

        info!("Finished status fetch, got capacity: {}", self.capacity);

        self.last_fetch = Instant::now();

        self.history
            .append(chrono::Utc::now().timestamp(), self.capacity);

        Ok(())
    }
}
