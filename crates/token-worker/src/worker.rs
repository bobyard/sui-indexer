use crate::aws::S3Store;
use anyhow::Result;
use diesel::PgConnection;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ExchangeKind};

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
        let _ = channel
            .queue_declare(
                "token::create",
                lapin::options::QueueDeclareOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;
        let _ = channel
            .queue_bind(
                "token::create",
                "token",
                "token",
                lapin::options::QueueBindOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;
        let _ = channel
            .exchange_declare(
                "collection",
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;
        let _ = channel
            .exchange_declare(
                "token",
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await;

        let mut consumer = channel
            .basic_consume(
                "token::create",
                "token",
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;
        while let Some(delivery) = consumer.next().await {
            let delivery = delivery.expect("error in consumer");

            dbg!(&delivery);
            let t = match serde_json::from_slice::<Token>(&delivery.data) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("error deserializing token: {}", e);
                    continue;
                }
            };

            delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }

        Ok(())
    }
}
