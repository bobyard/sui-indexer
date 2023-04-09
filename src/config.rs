use structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "sui-indexer")]
pub struct Config {
    #[structopt(short, long, default_value = "100")]
    pub max_size: u64,
    #[structopt(short, long, default_value = "http://localhost:9000")]
    pub node: String,
    #[structopt(short, long)]
    pub postgres: String,
    #[structopt(short, long, default_value = "redis://127.0.0.1/")]
    pub redis: String,
}
