use anyhow::Result;
use futures::future::{self};
use futures::StreamExt;
use lapin::options::BasicAckOptions;
use lapin::types::FieldTable;
use sui_indexer::indexer::receiver::TOKEN_EXCHANGE;
use sui_indexer::models::tokens::Token;
use tracing::info;

const TOKEN_CREATE: &str = "token.create";
const TOKEN_UPDATE: &str = "token.update";

pub async fn handle_token_create(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_CREATE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_CREATE,
            TOKEN_EXCHANGE,
            TOKEN_CREATE,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_CREATE,
            "server-side-token-create-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_UPDATE);

        let t = match serde_json::from_slice::<Token>(&delivery.data) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("error deserializing token: {}", e);
                continue;
            }
        };

        dbg!(&t);
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_update(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_UPDATE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_UPDATE,
            TOKEN_EXCHANGE,
            TOKEN_UPDATE,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_UPDATE,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer recver token update");

        let t = match serde_json::from_slice::<Token>(&delivery.data) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("error deserializing token: {}", e);
                continue;
            }
        };
        dbg!(&t);

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}
