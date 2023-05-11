use structopt::StructOpt;

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "sui-indexer")]
pub struct Config {
    #[structopt(short, long, default_value = "http://localhost:9000")]
    pub node: String,
    #[structopt(long)]
    pub postgres: String,
    #[structopt(long, default_value = "redis://127.0.0.1/")]
    pub redis: String,
    #[structopt(long)]
    pub bob_yard: String,
    #[structopt(long, default_value = "amqp://127.0.0.1:5672/%2f")]
    pub mq: String,
}
