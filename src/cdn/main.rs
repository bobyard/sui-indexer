mod aws;
mod runner;

use anyhow::Result;
use diesel::{Connection, PgConnection};

use dotenv::dotenv;

use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let filter = EnvFilter::from_default_env()
        .add_directive("mio=off".parse().unwrap())
        .add_directive("tokio_util=off".parse().unwrap());

    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut s3 = aws::S3Store::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut pg = PgConnection::establish(&database_url)?;
    loop {
        runner::run(&mut s3, &mut pg).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
