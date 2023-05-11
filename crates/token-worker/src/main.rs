use anyhow::Result;
use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use lapin::ConnectionProperties;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod aws;
mod token_worker;
mod worker;

use crate::worker::Worker;

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

    let rabbit_uri = std::env::var("RABBITMQ_URI").expect("RABBITMQ_URI must be set");
    let conn = lapin::Connection::connect(&rabbit_uri, ConnectionProperties::default()).await?;

    let mut worker = Worker::new(s3, pg, conn);
    worker.start().await
}
