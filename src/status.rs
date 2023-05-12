use {
    regex::Regex,
    reqwest::{Client, ClientBuilder},
    sqlx::{Pool, Postgres},
    std::{num::ParseIntError, time::Duration},
    tokio::time::interval,
    tracing::{error, info},
};

const URL: &'static str = "https://sport.wp.st-andrews.ac.uk/";

#[derive(Clone)]
pub struct StatusFetcher {
    db: Pool<Postgres>,
    client: Client,
    regex: Regex,
}

impl StatusFetcher {
    pub async fn init(db: Pool<Postgres>, period: Duration) {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let celf = Self {
            db,
            client,
            regex: Regex::new(r"Occupancy: ([0-9]+)%").unwrap(),
        };

        tokio::spawn(fetcher_task(celf, period));
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

        let capacity: u8 = percentage
            .parse()
            .map_err(|e| StatusUpdateError::Parse(e, percentage.to_owned()))?;

        info!("Finished status fetch, got capacity: {}", capacity);

        sqlx::query!(
            "INSERT INTO measurements (value) VALUES ($1)",
            i16::from(capacity),
        )
        .execute(&self.db)
        .await
        .unwrap();

        Ok(())
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
    /// Regex did not match response text {text}
    MissingCaptures { text: String },
    /// No capture group found at index {i} in text {text}
    MissingCaptureGroup { text: String, i: usize },
    /// Failed to parse {1:?} as u8: {0:?}
    Parse(ParseIntError, String),
}
