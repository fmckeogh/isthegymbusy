use {
    regex::{Match, Regex},
    reqwest::{Client, ClientBuilder, StatusCode},
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
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
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

        let response = self.client.get(URL).send().await?;

        if !response.status().is_success() {
            return Err(StatusUpdateError::Http(response.status()));
        }

        let text = response.text().await?;

        let captures = self
            .regex
            .captures(&text)
            .ok_or_else(|| StatusUpdateError::MissingCaptures)?;

        let percentage = captures.get(1).as_ref().map(Match::as_str).ok_or_else(|| {
            StatusUpdateError::MissingCaptureGroup {
                text: text.clone(),
                i: 1,
            }
        })?;

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
    /// Received HTTP error code {0}
    Http(StatusCode),
    /// Regex did not match response text
    MissingCaptures,
    /// No capture group found at index {i} in text {text}
    MissingCaptureGroup { text: String, i: usize },
    /// Failed to parse {1:?} as u8: {0:?}
    Parse(ParseIntError, String),
    /// Database error
    Database(#[from] sqlx::Error),
}
