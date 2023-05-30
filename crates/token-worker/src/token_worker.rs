use anyhow::Result;
use futures::future::join_all;

use crate::aws::S3Store;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicNackOptions};
use lapin::types::FieldTable;
use sui_indexer::indexer::receiver::TOKEN_EXCHANGE;
use sui_indexer::models::collections::{
    query_collection, update_collection_metadata,
};
use sui_indexer::models::tokens::count_star;
use sui_indexer::models::tokens::{update_image_url, Token};
use tracing::{error, info};

use crate::PgPool;

const TOKEN_CREATE: &str = "token.create";
const TOKEN_UPDATE: &str = "token.update";
const TOKEN_DELETE: &str = "token.delete";
const TOKEN_WRAP: &str = "token.wrap";
const TOKEN_UNWRAP: &str = "token.unwrap";
const TOKEN_UNWRAP_THEN_DELETE: &str = "token.unwrap_then_delete";

pub async fn batch_run_create_channel(
    batch: usize, mq: &lapin::Connection, pool: PgPool, s3: S3Store,
) -> Result<()> {
    let mut customers = vec![];

    for i in 0..batch {
        let channel = mq.create_channel().await?;

        customers.push(tokio::spawn(handle_token_create(
            i,
            channel,
            pool.clone(),
            s3.clone(),
        )));
    }

    let res = join_all(customers).await;
    for r in res {
        let _ = r?;
    }

    Ok(())
}

pub async fn handle_token_create(
    i: usize, channel: lapin::Channel, pool: PgPool, mut s3: S3Store,
) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_CREATE).await?;
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
        let mut t = match serde_json::from_slice::<Token>(&delivery.data) {
            Ok(t) => t,
            Err(e) => {
                error!("error deserializing token: {}", e);
                delivery
                    .nack(BasicNackOptions::default())
                    .await
                    .expect("nack");
                continue;
            }
        };

        //wait the db process done we can select and update
        //tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;

        // let url = t.metadata_uri
        // update to s3 store
        match s3.update_with_remote_url(t.metadata_uri.clone()).await {
            Ok(img_hash) => t.image = Some(img_hash),
            Err(e) => {
                error!("upload to aws err : {}", e.to_string());
                delivery
                    .nack(BasicNackOptions::default())
                    .await
                    .expect("nack");
                continue;
            }
        }
        let mut name = t.token_name.clone().trim().to_string();

        if !name.is_empty() {
            let mut display_name: Vec<_> = name.split("#").collect();
            if display_name.len() > 1 {
                display_name.remove(display_name.len() - 1);
                name = display_name.join(" ").to_string();
            }
        }

        //query the token numbers
        // let count =
        //     count_star(&mut pg, t.collection_id.clone()).unwrap_or_default();

        // query the collection
        if let Ok(mut collection) = query_collection(&mut pg, &t.collection_id)
        {
            if name.is_empty() {
                name = collection
                    .collection_name
                    .trim_end_matches(">")
                    .to_string();
            }

            if collection.display_name.is_none() {
                //give the display name with the nft name
                collection.display_name = Some(name.clone());
            }

            if collection.icon.is_none() {
                collection.icon = t.image.clone();
            }
            //collection.supply = count;
            if let Err(e) = update_collection_metadata(
                &mut pg,
                &t.collection_id,
                &collection,
            ) {
                error!("{}", e);
                delivery
                    .nack(BasicNackOptions::default())
                    .await
                    .expect("nack");
                continue;
            }
        }

        //info!("count collection_name: {} NFT-number: {}", name, count);
        if let Err(e) = update_image_url(&mut pg, t.token_id, t.image) {
            error!("{}", e);

            delivery
                .nack(BasicNackOptions::default())
                .await
                .expect("nack");
            continue;
        }

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_update(channel: lapin::Channel) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_UPDATE).await;

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

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_delete(
    channel: lapin::Channel, pool: PgPool,
) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_DELETE).await;

    let mut consumer = channel
        .basic_consume(
            TOKEN_DELETE,
            "server-side-token-update-worker",
            lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut pg = pool.get()?;

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

        //delete token
        //set_status_delete(&mut pg, &t.token_id)?;

        let mut name = t.token_name.clone();
        let mut display_name: Vec<_> = name.split("#").collect();
        if display_name.len() > 1 {
            display_name.remove(display_name.len() - 1);
            name = display_name.join(" ").to_string();
        }

        // query the collection
        let mut collection = query_collection(&mut pg, &t.collection_id)?;
        //query the token numbers
        let count = count_star(&mut pg, t.collection_id.clone())?;
        info!("count collection_name: {} NFT-number: {}", name, count);
        collection.supply = count;
        update_collection_metadata(&mut pg, &t.collection_id, &collection)?;

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_wrap(channel: lapin::Channel) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_WRAP).await;
    let mut consumer = channel
        .basic_consume(
            TOKEN_WRAP,
            "server-side-token-wrap-worker",
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

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_unwrap(channel: lapin::Channel) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_UNWRAP).await?;
    let mut consumer = channel
        .basic_consume(
            TOKEN_UNWRAP,
            "server-side-token-unwrap-worker",
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
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_unwrap_when_delete(
    channel: lapin::Channel,
) -> Result<()> {
    let _ = create_and_bind(&channel, &TOKEN_UNWRAP_THEN_DELETE).await;
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
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn create_and_bind(
    channel: &lapin::Channel, name: &str,
) -> Result<()> {
    let mut opt = lapin::options::QueueDeclareOptions::default();
    opt.durable = true;

    channel
        .queue_declare(name, opt, FieldTable::default())
        .await?;

    channel
        .queue_bind(
            name,
            TOKEN_EXCHANGE,
            name,
            lapin::options::QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(())
}
