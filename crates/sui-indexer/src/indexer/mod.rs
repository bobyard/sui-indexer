pub mod receiver;

use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use futures::future::join_all;
use redis::Commands;
use std::collections::HashMap;

use sui_sdk::types::messages_checkpoint::CheckpointSequenceNumber;
use sui_sdk::SuiClient;
use tokio::sync::mpsc::Sender;

use crate::models::activities::batch_insert as batch_insert_activities;
use crate::models::check_point::query_check_point;
use crate::{fetch_changed_objects, get_object_changes, multi_get_full_transactions, ObjectStatus};

use sui_sdk::rpc_types::{Checkpoint, SuiEvent, SuiObjectData, SuiTransactionBlockResponse};

use crate::config::Config;
use crate::handlers::activity::parse_tokens_activity;
use crate::handlers::bobyard_event_catch::{event_handle, parse_bob_yard_event};
use crate::handlers::collection::{collection_indexer_work, parse_collection};
use crate::handlers::token::{parse_tokens, token_indexer_work};
use crate::indexer::receiver::{IndexingMessage, Message};
use tracing::{debug, info};

extern crate redis;

use crate::schema::check_point::dsl::check_point;
use crate::schema::check_point::{chain_id, version};
use crate::MULTI_GET_CHUNK_SIZE;

pub(crate) struct Indexer {
    config: Config,
    sui_client: SuiClient,
    postgres: PgConnection,
    redis: redis::Client,
    sender: Sender<IndexingMessage>,
}

impl Indexer {
    pub fn new(
        config: Config,
        sui_client: SuiClient,
        postgres: PgConnection,
        redis: redis::Client,
        sender: Sender<IndexingMessage>,
    ) -> Self {
        Self {
            config,
            sui_client,
            postgres,
            redis,
            sender,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut redis = self.redis.get_connection()?;
        let mut indexer = query_check_point(&mut self.postgres, 1)? as u64;
        let mut collects_set: HashMap<String, String> = redis.hgetall("collections")?;

        loop {
            let (check_point_data, transactions, object_changed, events) =
                match self.download_checkpoint_data(indexer).await {
                    Ok(t) => t,
                    Err(e) => {
                        debug!(error=?e,"Got some error from node, we can continue to process");
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                };

            let collections = parse_collection(&object_changed, &mut redis, &mut collects_set)?;
            let tokens = parse_tokens(&object_changed, &mut collects_set)?;
            let bob_yard_events = parse_bob_yard_event(&events, &self.config.bob_yard)?;
            let token_activities = parse_tokens_activity(&bob_yard_events, &tokens);

            for (_, collection) in collections.iter() {
                self.sender
                    .send(IndexingMessage::Collection((
                        Message::Create,
                        collection.clone(),
                    )))
                    .await?;
            }

            for (_, t) in tokens.iter() {
                self.sender
                    .send(IndexingMessage::Token((Message::Create, t.0.clone())))
                    .await?;
            }

            // check_point_data.timestamp_ms
            self.postgres.build_transaction().read_write().run(|conn| {
                if collections.len() > 0 {
                    collection_indexer_work(&collections, conn).unwrap();
                }

                if tokens.len() > 0 {
                    token_indexer_work(&tokens, conn).unwrap();
                }

                if bob_yard_events.len() > 0 {
                    event_handle(&bob_yard_events, check_point_data.timestamp_ms as i64, conn)
                        .unwrap();
                }

                if token_activities.len() > 0 {
                    batch_insert_activities(conn, &token_activities).unwrap();
                }

                let updated_row = diesel::update(check_point.filter(chain_id.eq(1)))
                    .set(version.eq(indexer as i64))
                    .get_result::<(i64, i64)>(conn);

                assert_eq!(Ok((1, indexer as i64)), updated_row);
                Ok::<(), anyhow::Error>(())
            })?;

            info!(
                transactions = transactions.len(),
                indexer, "transactions processed"
            );
            indexer += 1;
        }
    }

    async fn download_checkpoint_data(
        &self,
        seq: CheckpointSequenceNumber,
    ) -> Result<(
        Checkpoint,
        Vec<SuiTransactionBlockResponse>,
        Vec<(ObjectStatus, SuiObjectData, String, u64)>,
        Vec<SuiEvent>,
    )> {
        let mut checkpoint = self.sui_client.read_api().get_checkpoint(seq.into()).await;

        while checkpoint.is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            debug!(
                "CheckPoint fetch failed, retrying... error: {:?}",
                checkpoint.unwrap_err()
            );
            checkpoint = self.sui_client.read_api().get_checkpoint(seq.into()).await;
        }

        // unwrap here is safe because we checked for error above
        let checkpoint = checkpoint.unwrap();

        let transactions = join_all(checkpoint.transactions.chunks(MULTI_GET_CHUNK_SIZE).map(
            |digests| {
                multi_get_full_transactions(self.sui_client.read_api().clone(), digests.to_vec())
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
            fetch_changed_objects(self.sui_client.read_api().clone(), object_changes)
                .await
                .map_err(|e| anyhow::format_err!("fetch_changed_objects err = {e}"))?;

        Ok((checkpoint, transactions, changed_objects, events))
    }
}
