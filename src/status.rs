use {
    regex::Regex,
    reqwest::{Client, ClientBuilder},
    sqlx::{Pool, Postgres},
    std::{
        num::ParseIntError,
        sync::{
            atomic::{AtomicU8, Ordering::Relaxed},
            Arc,
        },
        time::Duration,
    },
    tokio::time::interval,
    tracing::{error, info},
};

const URL: &str = "https://sport.wp.st-andrews.ac.uk/";

#[derive(Clone)]
pub struct StatusFetcher {
    capacity: Arc<AtomicU8>,
    db: Pool<Postgres>,
    client: Client,
    regex: Regex,
}

impl StatusFetcher {
    pub async fn init(db: Pool<Postgres>, period: Duration) -> Arc<AtomicU8> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (Android 4.4; Tablet; rv:41.0) Gecko/41.0 Firefox/41.0")
            .build()
            .unwrap();

        let capacity = Arc::new(AtomicU8::new(0));

        let celf = Self {
            capacity: capacity.clone(),
            db,
            client,
            regex: Regex::new(r"Occupancy: ([0-9]+)%").unwrap(),
        };

        tokio::spawn(fetcher_task_manager(celf, period));

        capacity
    }

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

        let capacity = percentage
            .parse()
            .map_err(|e| StatusUpdateError::Parse(e, percentage.to_owned()))?;

        self.capacity.store(capacity, Relaxed);

        info!("Finished status fetch, got capacity: {}", capacity);

        sqlx::query!(
            "INSERT INTO measurements (value) VALUES ($1)",
            i16::from(capacity),
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

async fn fetcher_task_manager(fetcher: StatusFetcher, period: Duration) {
    loop {
        let res = tokio::spawn(fetcher_task(fetcher.clone(), period)).await;
        error!("fetcher_task joined with result {:?}", res);
    }
}

async fn fetcher_task(mut fetcher: StatusFetcher, period: Duration) {
    let mut interval = interval(period);
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(e) = fetcher.update_status().await {
            error!("Error while updating status: {e:?}");
        }
    }
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
    /// Database error
    Database(#[from] sqlx::Error),
}
