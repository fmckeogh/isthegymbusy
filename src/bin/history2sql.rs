use {
    chrono::{DateTime, NaiveDateTime, Utc},
    color_eyre::eyre::Result,
    isthegymbusy::history::{Entry, PersistentHistory},
    sqlx::postgres::PgPoolOptions,
    std::{env::args, time::Duration},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let history_path = args().nth(1).unwrap();
    let database_url = args().nth(2).unwrap();

    let history = PersistentHistory::open(history_path);

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    let entries = history.get();

    for i in 0..entries.len() {
        let Entry { timestamp, value } = entries[i];

        if let Err(e) = sqlx::query!(
            "INSERT INTO measurements VALUES ($1, $2)",
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap(),
                Utc
            ),
            value as i32
        )
        .execute(&pool)
        .await
        {
            println!("{:?} {:?} {:?}", e, entries[i], entries[i - 1]);
        };
    }

    Ok(())
}
