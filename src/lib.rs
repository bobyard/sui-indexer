pub mod config;
pub mod handlers;
pub mod indexer;
pub mod models;
pub mod schema;
pub mod utils;

use sui_sdk::SuiClientBuilder;
use anyhow::{Error, Result};
use sui_sdk::rpc_types::{OwnedObjectRef, SuiGetPastObjectRequest, SuiObjectData, SuiObjectDataOptions, SuiTransactionBlockEffectsAPI,SuiTransactionBlockEffects, SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions};
use sui_sdk::types::digests::TransactionDigest;
use config::Config;
use indexer::Indexer;
use diesel::pg::PgConnection;
use diesel::Connection;
use futures::future::join_all;
use sui_sdk::apis::ReadApi;
use futures::FutureExt;


use sui_sdk::types::base_types::{ObjectID, SequenceNumber};
use tracing::info;

const MULTI_GET_CHUNK_SIZE: usize = 500;

#[derive(Debug, Clone, Copy)]
pub enum ObjectStatus {
    Created,
    Mutated,
    Deleted,
    Wrapped,
    Unwrapped,
    UnwrappedThenDeleted,
}

pub async fn run(cfg:Config)->Result<()> {
    let sui = SuiClientBuilder::default().build(
        &cfg.node,
    ).await?;

    let pg = PgConnection::establish(&cfg.postgres)?;
    let redis =  redis::Client::open(&*cfg.redis)?;

    Indexer::new(cfg,sui,pg,redis).start().await
}



pub async fn multi_get_full_transactions(
    http_client: &ReadApi,
    digests: Vec<TransactionDigest>,
) -> Result<Vec<SuiTransactionBlockResponse>> {
    let sui_transactions = http_client.multi_get_transactions_with_options(
        digests.clone(),
        SuiTransactionBlockResponseOptions::new()
            .with_object_changes()
            .with_input()
            .with_effects()
            .with_events()
            .with_raw_input(),
    ).await?;

    // let sui_full_transactions: Vec<_> = sui_transactions
    //     .into_iter()
    //     .collect::<Result<Vec<_>>>()?;

    Ok(sui_transactions)
}

pub fn get_object_changes(
    effects: &Option<SuiTransactionBlockEffects>,
) -> Vec<(ObjectID, SequenceNumber, ObjectStatus)> {
    if effects.is_none() {
        return vec![];
    }
    let effects = effects.clone().unwrap();

    let created = effects.created().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Created,
        )
    });
    let mutated = effects.mutated().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Mutated,
        )
    });
    let unwrapped = effects.unwrapped().iter().map(|o: &OwnedObjectRef| {
        (
            o.reference.object_id,
            o.reference.version,
            ObjectStatus::Unwrapped,
        )
    });
    created.chain(mutated).chain(unwrapped).collect()
}




pub async fn fetch_changed_objects(
    http_client: &ReadApi,
    object_changes: Vec<(ObjectID, SequenceNumber, ObjectStatus)>,
) -> Result<Vec<(ObjectStatus, SuiObjectData)>> {
    join_all(object_changes.chunks(MULTI_GET_CHUNK_SIZE).map(|objects| {
        let wanted_past_object_statuses: Vec<ObjectStatus> =
            objects.iter().map(|(_, _, status)| *status).collect();

        let wanted_past_object_request = objects
            .iter()
            .map(|(id, seq_num, _)| SuiGetPastObjectRequest {
                object_id: *id,
                version: *seq_num,
            })
            .collect();
        http_client
            .try_multi_get_parsed_past_object(
                wanted_past_object_request,
                SuiObjectDataOptions::bcs_lossless().with_content(),
            )
            .map(move |resp| (resp, wanted_past_object_statuses))
    }))
        .await
        .into_iter()
        .try_fold(vec![], |mut acc, chunk| {
            let object_datas = chunk.0?.into_iter().try_fold(vec![], |mut acc, resp| {
                let object_data = resp.into_object()?;
                acc.push(object_data);
                Ok::<Vec<SuiObjectData>, Error>(acc)
            })?;
            let mutated_object_chunk = chunk.1.into_iter().zip(object_datas);
            acc.extend(mutated_object_chunk);
            Ok::<_, Error>(acc)
        })
}