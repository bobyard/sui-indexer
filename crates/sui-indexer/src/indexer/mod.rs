pub mod receiver;

use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use futures::future::join_all;
use std::sync::Arc;

use futures::StreamExt;
use redis::Commands;
use std::collections::HashMap;

use sui_sdk::types::messages_checkpoint::CheckpointSequenceNumber;
use sui_sdk::SuiClient;
use tokio::sync::mpsc::Sender;

use crate::models::activities::{
    batch_insert as batch_insert_activities, Activity,
};
use crate::models::check_point::query_check_point;
use crate::{
    fetch_changed_objects, get_object_changes, multi_get_full_transactions,
    ObjectStatus,
};

use sui_sdk::rpc_types::{
    Checkpoint, SuiEvent, SuiObjectData, SuiTransactionBlockResponse,
};

use crate::config::Config;
use crate::handlers::activity::parse_tokens_activity;
use crate::handlers::bobyard_event_catch::{
    event_handle, parse_bob_yard_event, BobYardEvent,
};
use crate::handlers::collection::{
    collection_indexer_work, parse_collection, Collection,
};
use crate::handlers::token::{parse_tokens, token_indexer_work, Token};
use crate::indexer::receiver::IndexingMessage;
use tracing::{debug, info, warn};

extern crate redis;

use crate::schema::check_point::dsl::check_point;
use crate::schema::check_point::{chain_id, version};
use crate::MULTI_GET_CHUNK_SIZE;

const DOWNLOAD_RETRY_INTERVAL_IN_SECS: u64 = 10;

#[derive(Clone)]
pub(crate) struct Indexer {
    config: Config,
    sui_client: SuiClient,
    postgres: Pool<ConnectionManager<PgConnection>>,
    redis: redis::Client,
    sender: Sender<IndexingMessage>,
    check_point_data_sender: flume::Sender<Vec<CheckpointData>>,
    check_point_data_receiver: flume::Receiver<Vec<CheckpointData>>,
}

type CheckpointData = (
    Checkpoint,
    Vec<SuiTransactionBlockResponse>,
    Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    Vec<SuiEvent>,
);

impl Indexer {
    pub fn new(
        config: Config, sui_client: SuiClient,
        postgres: Pool<ConnectionManager<PgConnection>>, redis: redis::Client,
        sender: Sender<IndexingMessage>,
    ) -> Self {
        let (s, r) = flume::unbounded::<Vec<CheckpointData>>();

        Self {
            config,
            sui_client,
            postgres,
            redis,
            sender,
            check_point_data_sender: s,
            check_point_data_receiver: r,
        }
    }

    pub async fn run_forever(&mut self) -> Result<()> {
        let mut pg = self.postgres.get()?;
        let mut indexer = query_check_point(&mut pg, 1)? as u64;

        let batch_index = self.config.batch_index;

        loop {
            let download_futures = (indexer..(indexer + batch_index))
                .map(|x| download_checkpoint_data(&self.sui_client, x));

            let download_results = join_all(download_futures).await;
            let mut downloaded_checkpoints = vec![];

            for download_result in download_results {
                if let Ok(checkpoint) = download_result {
                    downloaded_checkpoints.push(checkpoint);
                } else {
                    if let Err(fn_e) = download_result {
                        warn!(
                            "Unexpected response from fullnode for checkpoints: {}",
                            fn_e
                        );
                    }
                    break;
                }
            }

            if downloaded_checkpoints.is_empty() {
                warn!(
                    "No checkpoints were downloaded for sequence number {}, retrying...",
                    indexer
                );
                tokio::time::sleep(std::time::Duration::from_secs(
                    DOWNLOAD_RETRY_INTERVAL_IN_SECS,
                ))
                .await;

                continue;
            }

            self.check_point_data_sender
                .send_async(downloaded_checkpoints.clone())
                .await?;

            indexer += downloaded_checkpoints.len() as u64;

            // let updated_row =
            //     diesel::update(check_point.filter(chain_id.eq(1)))
            //         .set(version.eq(indexer as i64))
            //         .get_result::<(i64, i64)>(&mut pg);
            //
            // assert_eq!(Ok((1, indexer as i64)), updated_row);

            let last_sequence = self
                .sui_client
                .read_api()
                .get_latest_checkpoint_sequence_number()
                .await?;

            info!(
                last_sequence = last_sequence,
                download_check_points = downloaded_checkpoints.len(),
                next_sequence_start = indexer,
                "transactions processed"
            );
        }
    }

    pub async fn handle_check_points(&mut self) -> Result<()> {
        let mut receiver = self.check_point_data_receiver.clone().into_stream();
        let mut pg = self.postgres.get()?;
        let mut redis = self.redis.get_connection()?;
        let mut collects_set: Arc<HashMap<String, String>> =
            Arc::new(redis.hgetall("collections")?);

        let bob_yard_contract = self.config.bob_yard.clone();

        while let Some(downloaded_checkpoints) = receiver.next().await {
            let parse_all_futures: Vec<_> = downloaded_checkpoints
                .into_iter()
                .map(|data| {
                    parse_all(
                        &bob_yard_contract,
                        redis,
                        collects_set.clone(),
                        data,
                    )
                })
                .collect();
        }

        Ok(())
    }

    // pub async fn shut_down(&self) {
    //     info!("Shutting down the index-workers...");
    //     // Abort the tasks.
    //
    //     //self.handles.lock().iter().for_each(|handle| handle.abort());
    //
    //     //self.tcp.shut_down().await;
    // }
}

async fn parse_all(
    bob_yard_contract: &String, rds: &mut redis::Connection,
    collects_set: Arc<HashMap<String, String>>,
    (check_point_datas, transactions, objects, events): CheckpointData,
) -> Result<(
    Vec<(ObjectStatus, Collection)>,
    Vec<(ObjectStatus, (Token, std::string::String))>,
    Vec<BobYardEvent>,
    Vec<Activity>,
)> {
    // for (check_point_data, _, object_changed, events) in
    //     downloaded_checkpoints
    // {

    let collections = parse_collection(&objects, rds, &mut collects_set)?;
    let tokens = parse_tokens(&objects, &mut collects_set)?;
    let bob_yard_events = parse_bob_yard_event(&events, &bob_yard_contract)?;
    let token_activities = parse_tokens_activity(&bob_yard_events, &tokens);

    Ok::<
        (
            Vec<(ObjectStatus, Collection)>,
            Vec<(ObjectStatus, (Token, std::string::String))>,
            Vec<BobYardEvent>,
            Vec<Activity>,
        ),
        anyhow::Error,
    >((collections, tokens, bob_yard_events, token_activities))
    // for (msg, collection) in collections.iter() {
    //     sender
    //         .send(IndexingMessage::Collection((
    //             (*msg).into(),
    //             collection.clone(),
    //         )))
    //         .await?;
    // }

    // for (msg, t) in tokens.iter() {
    //     self.sender
    //         .send(IndexingMessage::Token(((*msg).into(), t.0.clone())))
    //         .await?;
    // }

    // pg.build_transaction().read_write().run(|conn| {
    //     if collections.len() > 0 {
    //         collection_indexer_work(&collections, conn).unwrap();
    //     }

    //     if tokens.len() > 0 {
    //         token_indexer_work(&tokens, conn).unwrap();
    //     }

    //     if bob_yard_events.len() > 0 {
    //         event_handle(
    //             &bob_yard_events,
    //             check_point_data.timestamp_ms as i64,
    //             conn,
    //         )
    //         .unwrap();
    //     }

    //     if token_activities.len() > 0 {
    //         batch_insert_activities(conn, &token_activities).unwrap();
    //     }

    //     let updated_row = diesel::update(check_point.filter(chain_id.eq(1)))
    //         .set(version.eq(check_point_data.sequence_number as i64))
    //         .get_result::<(i64, i64)>(conn);
    //     assert_eq!(
    //         Ok((1, check_point_data.sequence_number as i64)),
    //         updated_row
    //     );

    //     info!(
    //         sequence_number = check_point_data.sequence_number,
    //         "indexer store success processed"
    //     );

    //     Ok::<(), anyhow::Error>(())
    // })?;
    //}
}
async fn download_checkpoint_data(
    sui_client: &SuiClient, seq: CheckpointSequenceNumber,
) -> Result<(
    Checkpoint,
    Vec<SuiTransactionBlockResponse>,
    Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    Vec<SuiEvent>,
)> {
    let mut checkpoint = sui_client.read_api().get_checkpoint(seq.into()).await;

    while checkpoint.is_err() {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        debug!(
            "CheckPoint fetch failed, retrying... error: {:?}",
            checkpoint.unwrap_err()
        );
        checkpoint = sui_client.read_api().get_checkpoint(seq.into()).await;
    }

    // unwrap here is safe because we checked for error above
    let checkpoint = checkpoint.unwrap();

    let transactions =
        join_all(checkpoint.transactions.chunks(MULTI_GET_CHUNK_SIZE).map(
            |digests| {
                multi_get_full_transactions(
                    sui_client.read_api().clone(),
                    digests.to_vec(),
                )
            },
        ))
        .await
        .into_iter()
        .try_fold(vec![], |mut acc, chunk| {
            acc.extend(chunk?);
            Ok::<Vec<SuiTransactionBlockResponse>, Error>(acc)
        })?;

    let mut object_changes = vec![];
    for tx in transactions.iter() {
        let new_object_changes = get_object_changes(tx)?;
        object_changes.extend(new_object_changes);
    }

    let mut events: Vec<SuiEvent> = vec![];
    for tx in transactions.iter() {
        if let Some(event) = &tx.events {
            events.extend(event.data.clone());
        }
    }

    let changed_objects =
        fetch_changed_objects(sui_client.read_api().clone(), object_changes)
            .await
            .map_err(|e| {
                anyhow::format_err!("fetch_changed_objects error = {e}")
            })?;

    Ok((checkpoint, transactions, changed_objects, events))
}
