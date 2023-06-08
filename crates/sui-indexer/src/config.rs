use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug, Default)]
#[structopt(name = "sui-indexer")]
pub struct Config {
    #[structopt(
        short,
        long,
        default_value = "http://localhost:9000",
        env = "FULLNODE"
    )]
    pub node: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub postgres: String,

    #[structopt(long, default_value = "redis://127.0.0.1/", env = "REDIS")]
    pub redis: String,

    #[structopt(long, env = "BOBYARD_CONTRACT")]
    pub bob_yard: String,

    #[structopt(long, env = "OB_CONTRACT")]
    pub origin_byte: String,

    #[structopt(
        long,
        default_value = "amqp://127.0.0.1:5672/%2f",
        env = "RABBITMQ_URI"
    )]
    pub mq: String,
    #[structopt(long, default_value = "25")]
    pub batch_index: u64,
}
