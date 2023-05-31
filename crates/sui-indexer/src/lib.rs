pub mod config;
pub mod handlers;
pub mod indexer;
pub mod models;
pub mod schema;
pub mod utils;

use anyhow::{anyhow, Error, Result};
use config::Config;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use futures::future::join_all;
use futures::FutureExt;
use indexer::Indexer;
use lapin::ConnectionProperties;
use sui_sdk::apis::ReadApi;
use sui_sdk::rpc_types::SuiTransactionBlockData::V1;
use sui_sdk::rpc_types::{
    OwnedObjectRef, SuiGetPastObjectRequest, SuiObjectData,
    SuiObjectDataOptions, SuiTransactionBlockEffectsAPI,
    SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
};
use sui_sdk::types::digests::TransactionDigest;
use sui_sdk::SuiClientBuilder;

use crate::indexer::receiver::{IndexSender, IndexingMessage};
use sui_sdk::types::base_types::{ObjectID, SequenceNumber};

const MULTI_GET_CHUNK_SIZE: usize = 500;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ObjectStatus {
    Created,
    Mutated,
    Deleted,
    Wrapped,
    Unwrapped,
    UnwrappedThenDeleted,
}

pub async fn run(cfg: Config) -> Result<()> {
    let sui = SuiClientBuilder::default()
        .build(&cfg.node)
        .await
        .map_err(|e| anyhow!("Pg: {e}"))?;
    // let pg = PgConnection::establish(&cfg.postgres)
    //     .map_err(|e| anyhow!("Pg: {e}"))?;

    let manager = ConnectionManager::<PgConnection>::new(&cfg.postgres);

    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let redis = redis::Client::open(&*cfg.redis)?;
    let conn =
        lapin::Connection::connect(&cfg.mq, ConnectionProperties::default())
            .await?;

    let (send, recv) = tokio::sync::mpsc::channel::<IndexingMessage>(1000);
    tokio::spawn(async move {
        let mut receiver = IndexSender::new(recv, conn);
        receiver
            .process()
            .await
            .expect("Unexpected error in receiver");
    });

    let mut index = Indexer::new(cfg, sui, pool, redis, send);
    let mut handle = index.clone();
    tokio::spawn(async move { handle.handle_check_points().await });

    index.run_forever().await
}

pub async fn multi_get_full_transactions(
    http_client: &ReadApi, digests: Vec<TransactionDigest>,
) -> Result<Vec<SuiTransactionBlockResponse>> {
    let sui_transactions = http_client
        .multi_get_transactions_with_options(
            digests.clone(),
            SuiTransactionBlockResponseOptions::new()
                .with_object_changes()
                .with_input()
                .with_effects()
                .with_events()
                .with_raw_input(),
        )
        .await?;
    Ok(sui_transactions)
}

pub fn get_object_changes(
    block: &SuiTransactionBlockResponse,
) -> Result<Vec<(ObjectID, SequenceNumber, ObjectStatus, String, u64)>> {
    let effects = match block.effects.clone() {
        Some(effects) => effects,
        None => anyhow::bail!("No effects in block"),
    };

    let transaction = match block.transaction.clone() {
        Some(transaction) => match transaction.data {
            V1(v1) => v1,
        },
        _ => return Err(anyhow!("Transaction is not V1")),
    };

    match block.timestamp_ms {
        Some(_) => (),
        None => return Err(anyhow!("No timestamp in block")),
    }

    let created = effects.created().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Created,
            transaction.sender.to_string(),
            block.timestamp_ms.unwrap(),
        )
    });

    let mutated = effects.mutated().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Mutated,
            transaction.sender.to_string(),
            block.timestamp_ms.unwrap(),
        )
    });

    let unwrapped = effects.unwrapped().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Unwrapped,
            transaction.sender.to_string(),
            block.timestamp_ms.unwrap(),
        )
    });

    Ok(created.chain(mutated).chain(unwrapped).collect())
}

pub async fn fetch_changed_objects(
    http_client: &ReadApi,
    object_changes: Vec<(ObjectID, SequenceNumber, ObjectStatus, String, u64)>,
) -> Result<Vec<(ObjectStatus, SuiObjectData, String, u64)>> {
    join_all(object_changes.chunks(MULTI_GET_CHUNK_SIZE).map(|objects| {
        let wanted_past_object_statuses: Vec<ObjectStatus> =
            objects.iter().map(|(_, _, status, _, _)| *status).collect();
        let senders: Vec<String> = objects
            .iter()
            .map(|(_, _, _, sender, _)| sender.clone())
            .collect();
        let times: Vec<u64> =
            objects.iter().map(|(_, _, _, _, t)| *t).collect();

        let wanted_past_object_request = objects
            .iter()
            .map(|(id, seq_num, _, _, _)| SuiGetPastObjectRequest {
                object_id: *id,
                version: *seq_num,
            })
            .collect();
        http_client
            .try_multi_get_parsed_past_object(
                wanted_past_object_request,
                SuiObjectDataOptions::bcs_lossless()
                    .with_content()
                    .with_display(),
            )
            .map(move |resp| {
                (resp, wanted_past_object_statuses, senders, times)
            })
    }))
    .await
    .into_iter()
    .try_fold(vec![], |mut acc, chunk| {
        let object_datas =
            chunk.0?.into_iter().try_fold(vec![], |mut acc, resp| {
                // let object_data = if let Ok(obj) = resp.into_object() {
                //     Some(obj)
                // } else {
                //     error!("Failed to parse object data,local fullnode have
                // not full-data");     None
                // };
                let object_data = resp.into_object()?;

                acc.push(object_data);
                //Ok::<Vec<Option<SuiObjectData>>, Error>(acc)
                Ok::<Vec<SuiObjectData>, Error>(acc)
            })?;

        let mutated_object_chunk: Vec<_> = chunk
            .1
            .into_iter()
            .zip(object_datas)
            .zip(chunk.2)
            .zip(chunk.3)
            .collect();
        let mutated_object_chunk: Vec<(
            ObjectStatus,
            SuiObjectData,
            String,
            u64,
        )> = mutated_object_chunk
            .into_iter()
            // .filter_map(|(((status, obj), sender), timestamp)|{
            //     if obj.is_some() {
            //         Some((status, obj.unwrap(), sender, timestamp))
            //     } else {
            //         None
            //     }
            // })
            .map(|(((status, obj), sender), timestamp)| {
                (status, obj, sender, timestamp)
            })
            .collect();

        //let mutated_object_chunk:Vec<(ObjectStatus,
        // SuiObjectData,String,u64)> =
        // mutated_object_chunk.iter().map(|(status,obj)|
        // (status,obj,chunk.2.clone(),chunk.3)).collect();
        // let mut mutated_object_chunk =
        // mutated_object_chunk.into_iter().zip(chunk.2).collection();
        // let mut mutated_object_chunk =
        // mutated_object_chunk.into_iter().zip(chunk.3).collection();

        acc.extend(mutated_object_chunk);
        Ok::<_, Error>(acc)
    })
}
