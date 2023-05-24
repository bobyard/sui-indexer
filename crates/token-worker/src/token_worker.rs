use anyhow::Result;
use futures::future::join_all;

use crate::aws::S3Store;
use futures::StreamExt;
use lapin::options::BasicAckOptions;
use lapin::types::FieldTable;
use sui_indexer::indexer::receiver::TOKEN_EXCHANGE;
use sui_indexer::models::collections::{
    query_collection, update_collection_metadata,
};
use sui_indexer::models::tokens::{update_image_url, Token};
use tracing::{debug, info};

use crate::PgPool;

const TOKEN_CREATE: &str = "token.create";
const TOKEN_UPDATE: &str = "token.update";
const TOKEN_DELETE: &str = "token.delete";
const TOKEN_WRAP: &str = "token.wrap";
const TOKEN_UNWRAP: &str = "token.unwrap";
const TOKEN_UNWRAP_THEN_DELETE: &str = "token.unwrap_then_delete";

pub async fn batch_run_create_channel(
    batch: usize, channel: lapin::Channel, pool: PgPool, s3: S3Store,
) -> Result<()> {
    let mut customers = vec![];

    for i in 0..batch {
        customers.push(handle_token_create(
            i,
            channel.clone(),
            pool.clone(),
            s3.clone(),
        ));
    }

    let res = join_all(customers).await;
    for r in res {
        r?;
    }
    Ok(())
}

pub async fn handle_token_create(
    i: usize, channel: lapin::Channel, pool: PgPool, mut s3: S3Store,
) -> Result<()> {
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
            format!("server-side-token-create-worker-{}", i).as_str(),
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut pg = pool.get()?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_CREATE);

        info!("{}", String::from_utf8(delivery.data.clone()).unwrap());

        let mut t = match serde_json::from_slice::<Token>(&delivery.data) {
            Ok(t) => t,
            Err(e) => {
                info!("error deserializing token: {}", e);
                continue;
            }
        };

        //wait the db process done we can select and update
        //tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;

        // let url = t.metadata_uri
        // update to s3 store
        match s3.update_with_remote_url(t.metadata_uri.clone()).await {
            Ok(img_hash) => t.image = Some(img_hash),
            Err(e) => info!("upload to aws err : {}", e.to_string()),
        }

        // query the collection
        let mut collection = query_collection(&mut pg, &t.collection_id)?;
        if collection.display_name.is_none() {
            //give the display name with the nft name
            let name = t.token_name.clone();
            let display_name: Vec<_> = name.split("#").collect();

            if display_name.len() > 1 {
                collection.display_name =
                    Some(display_name.get(0).unwrap().to_string());
            } else {
                collection.display_name = Some(name.clone());
            }
        }

        if collection.icon.is_none() {
            collection.icon = t.image.clone();
        }

        update_collection_metadata(&mut pg, &t.collection_id, &collection)?;
        //let change = vec![t];
        //batch_change(&mut pg, &change)?;
        update_image_url(&mut pg, t.token_id, t.image)?;
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

pub async fn handle_token_delete(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_DELETE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_DELETE,
            TOKEN_EXCHANGE,
            TOKEN_DELETE,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_DELETE,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_DELETE);

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

pub async fn handle_token_wrap(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_WRAP,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_WRAP,
            TOKEN_EXCHANGE,
            TOKEN_WRAP,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_WRAP,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_WRAP);

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

pub async fn handle_token_unwrap(channel: lapin::Channel) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_UNWRAP,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_UNWRAP,
            TOKEN_EXCHANGE,
            TOKEN_UNWRAP,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_UNWRAP,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_UNWRAP);

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

pub async fn handle_token_unwrap_when_delete(
    channel: lapin::Channel,
) -> Result<()> {
    channel
        .queue_declare(
            TOKEN_UNWRAP_THEN_DELETE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            TOKEN_UNWRAP_THEN_DELETE,
            TOKEN_EXCHANGE,
            TOKEN_UNWRAP_THEN_DELETE,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            TOKEN_UNWRAP_THEN_DELETE,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        info!("consumer: {}", TOKEN_UNWRAP_THEN_DELETE);

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
