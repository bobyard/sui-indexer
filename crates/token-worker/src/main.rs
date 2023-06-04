use anyhow::Result;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use lapin::ConnectionProperties;
use sui_indexer::models::collections::Collection;
use token_worker::aws;
use token_worker::worker::Worker;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new("info"))
        .add_directive("mio=off".parse().unwrap())
        .add_directive("tokio_util=off".parse().unwrap());

    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let s3 = aws::S3Store::new();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager =
        ConnectionManager::<diesel::pg::PgConnection>::new(database_url);

    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let redis = redis::Client::open(
        &*std::env::var("REDIS").expect("REDIS must be set"),
    )?;
    let rabbit_uri =
        std::env::var("RABBITMQ_URI").expect("RABBITMQ_URI must be set");
    let conn = lapin::Connection::connect(
        (&rabbit_uri).as_ref(),
        ConnectionProperties::default(),
    )
    .await?;

    // read ALGOLIA_APPLICATION_ID and ALGOLIA_API_KEY from env
    // let algo = algoliasearch::Client::new(
    //     "K6MYR2JP0U",
    //     "2f820aa6c2ba05b1ea20abdd951e4ca7",
    // );
    // let activities_index = algo.init_index::<Activity>("activities");

    let index = algoliasearch::Client::new(
        &std::env::var("ALGOLIA_APPLICATION_ID")
            .expect("ALGOLIA_APPLICATION_ID must be set"),
        &std::env::var("ALGOLIA_API_KEY").expect("ALGOLIA_API_KEY must be set"),
    )
    .init_index::<Collection>("collections");

    let mut worker = Worker::new(s3, pool, conn, redis, index);
    worker.start().await
}
