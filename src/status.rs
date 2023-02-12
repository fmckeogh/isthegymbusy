use {
    crate::{
        config,
        history::{Entry, PersistentHistory},
    },
    futures::lock::Mutex,
    regex::Regex,
    reqwest::Client,
    std::{
        sync::Arc,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
    tokio::time::sleep,
    tracing::info,
};

const URL: &'static str = "https://sport.wp.st-andrews.ac.uk/";

#[derive(Clone)]
pub struct StatusFetcher(Arc<Mutex<Inner>>);

impl StatusFetcher {
    pub async fn new() -> Self {
        let inner = Arc::new(Mutex::new(Inner {
            capacity: 0,
            last_fetch: Instant::now(),
            client: Client::new(),
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
    loop {
        inner.lock().await.update_status().await;
        sleep(Duration::from_secs(config::get().fetch_interval)).await;
    }
}

pub struct Inner {
    capacity: u8,
    last_fetch: Instant,
    client: Client,
    regex: Regex,
    history: PersistentHistory,
}

impl Inner {
    async fn update_status(&mut self) {
        info!("Starting status fetch");

        let res = self.client.get(URL).send().await.unwrap();

        let text = res.text().await.unwrap();

        self.capacity = self
            .regex
            .captures(&text)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .unwrap();

        info!("Finished status fetch, got capacity: {}", self.capacity);

        self.last_fetch = Instant::now();

        self.history.append(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.capacity,
        );
    }
}
