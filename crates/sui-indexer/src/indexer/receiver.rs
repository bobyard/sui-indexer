use crate::models::collections::Collection;
use crate::models::tokens::Token;
use anyhow::anyhow;
use image::EncodableLayout;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, ExchangeKind, Result,
};
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tracing::info;

pub enum Message {
    Create,
    Delete,
    Update,
    Wrap,
    Unwrap,
    UnwrapThenDelete,
}

pub enum IndexingMessage {
    Collection((Message, Collection)),
    Token((Message, Token)),
}

pub struct IndexSender {
    receiver: Receiver<IndexingMessage>,
    rabbitmq: Connection,
}

pub const TOKEN_EXCHANGE: &str = "token";
pub const COLLECTION_EXCHANGE: &str = "collection";

impl IndexSender {
    pub fn new(receiver: Receiver<IndexingMessage>, conn: Connection) -> Self {
        Self {
            receiver,
            rabbitmq: conn,
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        let mut channel = self.rabbitmq.create_channel().await?;
        let _ = match create_exchange(channel).await {
            Ok(_) => info!("exchange created"),
            Err(e) => info!("error creating exchange: {}", e),
        };

        let mut channel = self.rabbitmq.create_channel().await?;

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                IndexingMessage::Collection((message, collection)) => {
                    info!("Collection: {:?}", &collection);
                    let payload = serde_json::to_vec(&collection)
                        .expect("send collection to json failed")
                        .clone();
                    channel
                        .basic_publish(
                            "",
                            "collection",
                            BasicPublishOptions::default(),
                            &payload,
                            BasicProperties::default(),
                        )
                        .await?;
                }
                IndexingMessage::Token((message, token)) => {
                    let payload = serde_json::to_vec(&token)
                        .expect("send collection to json failed")
                        .clone();
                    channel
                        .basic_publish(
                            "",
                            "token.*",
                            BasicPublishOptions::default(),
                            &payload,
                            BasicProperties::default(),
                        )
                        .await?;
                    info!("Token: {:?}", token);
                }
            }
        }

        Ok(())
    }
}

pub async fn create_exchange(channel: lapin::Channel) -> Result<()> {
    let mut opt = ExchangeDeclareOptions::default();
    opt.durable = true;

    let _ = channel
        .exchange_declare(
            COLLECTION_EXCHANGE,
            ExchangeKind::Topic,
            opt,
            FieldTable::default(),
        )
        .await?;

    let mut opt = ExchangeDeclareOptions::default();
    opt.durable = true;
    let _ = channel
        .exchange_declare(
            TOKEN_EXCHANGE,
            ExchangeKind::Topic,
            opt,
            FieldTable::default(),
        )
        .await?;

    channel.clone();
    Ok(())
}
