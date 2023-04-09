use anyhow::Result;
use sui_indexer;


use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env()
        .add_directive("mio=off".parse().unwrap())
        .add_directive("tokio_util=off".parse().unwrap())
        // .add_directive("hyper=off".parse().unwrap())
        // .add_directive("hyper::proto::h1::decode=off".parse().unwrap())
        // .add_directive("hyper::proto::h1::io=off".parse().unwrap())
        // .add_directive("hyper::proto::h1::role=off".parse().unwrap())
        .add_directive("jsonrpsee=off".parse().unwrap());

    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(filter)
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = sui_indexer::config::init()?;
    if let Err(e) = sui_indexer::run(config).await {
        panic!("Error: {}", e);
    }

    Ok(())
}
