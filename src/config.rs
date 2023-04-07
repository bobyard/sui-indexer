#[derive(Debug, Default)]
pub struct Config {
    pub max_size: u64,
    pub node: String,
    pub postgres: String,
}

pub fn init() -> anyhow::Result<Config>{
    let mut c = Config::default();
    c.max_size = 100;
    c.node = "http://localhost:9000".to_string();
    c.postgres = "postgres://sui:woaini521@localhost/sui".to_string();

    Ok(c)
}