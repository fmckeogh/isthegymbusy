use {
    color_eyre::eyre::Result,
    isthegymbusy::{start, Config},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    start(&Config::new()?).await?.join().await
}
