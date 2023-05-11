use anyhow::Result;
use futures::StreamExt;
use lapin::options::BasicAckOptions;
use lapin::types::FieldTable;
use sui_indexer::models::tokens::Token;

const NAME: &str = "token.create";

pub async fn handle_token_create(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            NAME,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            NAME,
            "token",
            NAME,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            NAME,
            "server-side-token-create-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
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
