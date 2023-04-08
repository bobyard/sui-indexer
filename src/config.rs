#[derive(Debug, Default)]
pub struct Config {
    pub max_size: u64,
    pub node: String,
    pub postgres: String,
    pub redis:String,
}

pub fn init() -> anyhow::Result<Config>{
    let mut c = Config::default();
    c.max_size = 100;
    c.node = "http://localhost:9000".to_string();
    c.postgres = "postgres://bobyard:!Hacker42979127200@localhost/bobyard".to_string();
    c.redis = "redis://127.0.0.1/".to_string();

    Ok(c)
}