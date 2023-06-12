use anyhow::{anyhow, Result};
use futures::future::join_all;
use redis::Commands;

use crate::aws::S3Store;
use crate::PgPool;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicNackOptions, BasicQosOptions};
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};
use sui_indexer::indexer::receiver::TOKEN_EXCHANGE;
use sui_indexer::models::collections::Collection;
use sui_indexer::models::collections::{
    query_collection, update_collection_metadata,
};
use sui_indexer::models::tokens::count_star;
use sui_indexer::models::tokens::{update_image_url, Token};
use tracing::{error, info};

const TOKEN_CREATE: &str = "token.create";
const TOKEN_UPDATE: &str = "token.update";
const TOKEN_DELETE: &str = "token.delete";
const TOKEN_WRAP: &str = "token.wrap";
const TOKEN_UNWRAP: &str = "token.unwrap";
const TOKEN_UNWRAP_THEN_DELETE: &str = "token.unwrap_then_delete";

#[derive(Deserialize, Serialize, Debug)]
struct CollectionObjectId {
    object_id: String,
    collection_id: String,
}

pub async fn batch_run_create_channel(
    batch: usize,
    mq: lapin::Connection,
    pool: PgPool,
    s3: S3Store,
    rds: redis::Client,
) -> Result<()> {
    let mut customers = vec![];

    for i in 0..batch {
        let channel = mq.create_channel().await?;
        channel.basic_qos(1, BasicQosOptions::default()).await?;

        let r = rds.clone();
        customers.push(tokio::spawn(handle_token_create(
            i,
            channel,
            pool.clone(),
            s3.clone(),
            r,
        )));
    }

    let res = join_all(customers).await;
    for r in res {
        let _ = r?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct JsonMetaData {
    description: String,
}

pub async fn handle_token_create(
    i: usize,
    channel: lapin::Channel,
    pool: PgPool,
    mut s3: S3Store,
    mut rds: redis::Client,
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

    let collection_index_write = algoliasearch::Client::new(
        &std::env::var("ALGOLIA_APPLICATION_ID")
            .expect("ALGOLIA_APPLICATION_ID must be set"),
        &std::env::var("ALGOLIA_API_KEY").expect("ALGOLIA_API_KEY must be set"),
    )
    .init_index::<Collection>("collections");

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
        let mut pg = pool.get()?;

        let mut nack = BasicNackOptions::default();
        nack.requeue = !delivery.redelivered;

        let cache: Option<String> =
            rds.hget("url_caches", t.metadata_uri.clone())?;
        if cache.is_none() {
            match s3.update_with_remote_url(t.metadata_uri.clone()).await {
                Ok(img_hash) => {
                    let _: () = rds
                        .hset(
                            "url_caches",
                            t.metadata_uri.clone(),
                            img_hash.clone(),
                        )
                        .unwrap();
                    t.image = Some(img_hash)
                }
                Err(e) => {
                    error!("upload to aws err : {}", e.to_string());
                    delivery.nack(nack).await.expect("nack");
                    continue;
                }
            }
        } else {
            t.image = Some(cache.unwrap())
        };

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
            let mut insert_to_searche_ngine = false;

            if name.is_empty() {
                name = collection
                    .collection_name
                    .trim_end_matches(">")
                    .to_string();
            }

            if collection.display_name.is_none() {
                collection.display_name = Some(name.clone());
                insert_to_searche_ngine = true;
            }

            if collection.icon.is_none() {
                collection.icon = t.image.clone();
                //collection.description=t.metadata_uri
                collection.description = if let Some(metadata) = t.metadata_json
                {
                    let a = serde_json::from_str::<JsonMetaData>(&metadata)
                        .unwrap_or_default();

                    a.description
                } else {
                    "".to_string()
                }
            }

            // let collection_search = collection_index_read
            //     .search(collection.collection_id.as_str())
            //     .await
            //     .map_err(|e| {
            //         anyhow!(format!(
            //             "Failed search collection_id {} error {:?}",
            //             collection.collection_id, e
            //         ))
            //     })?;

            // dbg!(&collection_search);
            // if collection_search.hits.len() == 1 {
            //     for c in collection_search.hits {
            //         collection_index_write
            //             .update_object(&collection, &c.object_id)
            //             .await
            //             .map_err(|e| {
            //                 anyhow!(format!(
            //                     "Failed update collection_id {} error {:?}",
            //                     collection.collection_id, e
            //                 ))
            //             })?;
            //     }
            // } else {
            //     for c in collection_search.hits {
            //         collection_index_write
            //             .delete_object(&c.object_id)
            //             .await
            //             .map_err(|e| {
            //                 anyhow!(format!(
            //                     "Failed delete collection_id {} error {:?}",
            //                     collection.collection_id, e
            //                 ))
            //             })?;
            //     }

            //     collection_index_write
            //         .add_object(&collection)
            //         .await
            //         .map_err(|e| {
            //             anyhow!(format!(
            //                 "Failed add upgrand collection_id {} error {:?}",
            //                 collection.collection_id, e
            //             ))
            //         })?;
            // }

            if insert_to_searche_ngine {
                collection_index_write
                    .add_object(&collection)
                    .await
                    .map_err(|e| {
                        anyhow!(format!(
                            "Failed add upgrand collection_id {} error {:?}",
                            collection.collection_id, e
                        ))
                    })?;

                info!("Success insert the query collection to search engine");
            }

            if let Err(e) = update_collection_metadata(
                &mut pg,
                &t.collection_id,
                &collection,
            ) {
                error!("{}", e);
                delivery.nack(nack).await.expect("nack");
                continue;
            }
        }

        //info!("count collection_name: {} NFT-number: {}", name, count);
        if let Err(e) = update_image_url(&mut pg, t.token_id, t.image) {
            error!("{}", e);

            delivery.nack(nack).await.expect("nack");
            continue;
        }

        //set to redis,

        //when update read from redis, and check is token or not?

        //if is token we keep know what he doing.

        info!("Success for create token return ask to channel");

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
        //info!("consumer: {}", TOKEN_UPDATE);

        // let _t = match serde_json::from_slice::<Token>(&delivery.data) {
        //     Ok(t) => t,
        //     Err(e) => {
        //         tracing::error!("error deserializing token: {}", e);
        //         continue;
        //     }
        // };

        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }

    Ok(())
}

pub async fn handle_token_delete(
    channel: lapin::Channel,
    pool: PgPool,
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

        // info!("consumer: {}", TOKEN_WRAP);

        // let _t = match serde_json::from_slice::<Token>(&delivery.data) {
        //     Ok(t) => t,
        //     Err(e) => {
        //         tracing::error!("error deserializing token: {}", e);
        //         continue;
        //     }
        // };

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
        //info!("consumer: {}", TOKEN_UNWRAP);

        // let _t = match serde_json::from_slice::<Token>(&delivery.data) {
        //     Ok(t) => t,
        //     Err(e) => {
        //         tracing::error!("error deserializing token: {}", e);
        //         continue;
        //     }
        // };

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

        let _t = match serde_json::from_slice::<Token>(&delivery.data) {
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
    channel: &lapin::Channel,
    name: &str,
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
