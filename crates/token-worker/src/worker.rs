use crate::aws::S3Store;
use anyhow::Result;
use diesel::PgConnection;
use futures::future::join_all;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ExchangeKind};
use sui_indexer::indexer::receiver::create_exchange;

use crate::token_worker::handle_token_create;
use sui_indexer::models::tokens::Token;

pub struct Worker {
    s3: S3Store,
    pg: PgConnection,
    mq: Connection,
}

impl Worker {
    pub fn new(s3: S3Store, pg: PgConnection, mq: Connection) -> Self {
        Worker { s3, pg, mq }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut channel = self.mq.create_channel().await?;
        let _ = match create_exchange(channel).await {
            Ok(_) => tracing::info!("exchange created"),
            Err(e) => tracing::info!("error creating exchange: {}", e),
        };

        let mut channel = self.mq.create_channel().await?;
        //tokio::spawn(handle_create(channel));
        handle_token_create(channel).await?;
        //join_all(vec![handle_create(&mut channel)]).await?;

        Ok(())
    }
}
