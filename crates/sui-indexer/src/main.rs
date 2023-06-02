use anyhow::Result;
use dotenv;
use structopt::StructOpt;
use sui_indexer;

use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

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
    let config = sui_indexer::config::Config::from_args();
    if let Err(e) = sui_indexer::run(config).await {
        panic!("Error: {}", e);
    }

    Ok(())
}
