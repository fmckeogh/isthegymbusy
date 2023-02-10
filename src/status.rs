use {
    futures::lock::Mutex,
    regex::Regex,
    reqwest::Client,
    std::{
        sync::Arc,
        time::{Duration, Instant},
    },
    tracing::trace,
};

const URL: &'static str = "https://sport.wp.st-andrews.ac.uk/";

#[derive(Clone)]
pub struct Status(Arc<Mutex<Inner>>);

impl Status {
    pub async fn new(validity: Duration) -> Self {
        let mut inner = Inner {
            capacity: 0,
            last_fetch: Instant::now(),
            validity,
            client: Client::new(),
            regex: Regex::new(r"Occupancy: ([0-9]+)%").unwrap(),
        };

        inner.update_status().await;

        Self(Arc::new(Mutex::new(inner)))
    }

    pub async fn get(&mut self) -> u8 {
        let mut inner = self.0.lock().await;

        if inner.last_fetch.elapsed() > inner.validity {
            inner.update_status().await;
        };

        inner.capacity
    }
}

pub struct Inner {
    capacity: u8,
    last_fetch: Instant,
    validity: Duration,
    client: Client,
    regex: Regex,
}

impl Inner {
    async fn update_status(&mut self) {
        trace!("fetching current status");
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

        trace!("got capacity: {}", self.capacity);

        self.last_fetch = Instant::now();
    }
}
