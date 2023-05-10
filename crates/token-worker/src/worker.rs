use crate::aws::S3Store;
use anyhow::Result;
use diesel::PgConnection;
use futures::StreamExt;
use lapin::options::BasicAckOptions;
use lapin::Connection;

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
        let mut consumer = channel
            .basic_consume(
                "token",
                "token",
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");
            dbg!(&delivery);
            //let t = serde_json::from_str::<Token>(&delivery);

            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }

        Ok(())
    }
}
