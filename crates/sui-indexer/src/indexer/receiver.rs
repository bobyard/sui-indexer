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

impl IndexSender {
    pub fn new(receiver: Receiver<IndexingMessage>, conn: Connection) -> Self {
        Self {
            receiver,
            rabbitmq: conn,
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        let channel = self.rabbitmq.create_channel().await?;

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

        while let Some(msg) = self.receiver.recv().await {
            match msg {
                IndexingMessage::Collection((message, collection)) => {
                    info!("Collection: {:?}", &collection);
                    let payload = serde_json::to_vec(&collection)
                        .expect("serd collection to json failed")
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
                        .expect("serd collection to json failed")
                        .clone();
                    channel
                        .basic_publish(
                            "",
                            "token::*",
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
