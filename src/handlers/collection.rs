use crate::config::Config;
use anyhow::Result;

pub struct CollectionHandler<'a> {
    pub cfg:&'a Config,
    pub postgres: String,
}


impl<'a> CollectionHandler<'a> {
    pub fn new(cfg:&'a Config) -> Self {
        Self {
            cfg,
            postgres: cfg.postgres.clone(),
        }
    }
}


impl CollectionHandler<'_> {
    pub async fn start() -> Result<()>{
        //todo insert to db
        Ok(())
    }
}