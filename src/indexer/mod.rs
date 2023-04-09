use crate::config::Config;
use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use sui_sdk::types::messages_checkpoint::CheckpointSequenceNumber;
use sui_sdk::SuiClient;

use crate::models::activities::{batch_insert as batch_insert_activities, Activity, ActivityType};
use crate::{fetch_changed_objects, get_object_changes, multi_get_full_transactions, ObjectStatus};
use chrono::prelude::*;
use redis::Commands;
use serde_json::value::Index;
use serde_json::Value;
use sui_sdk::rpc_types::{Checkpoint, SuiObjectData, SuiParsedData, SuiTransactionBlockResponse};
use sui_sdk::types::object::Object;
use tracing::{debug, info};

use crate::models::collections::{batch_insert, Collection};
use crate::models::tokens::{
    batch_change as batch_change_tokens, batch_insert as batch_insert_tokens, Token,
};

extern crate redis;

use crate::MULTI_GET_CHUNK_SIZE;

pub(crate) struct Indexer {
    cfg: Config,
    sui_client: SuiClient,
    postgres: PgConnection,
    redis: redis::Client,
}

impl Indexer {
    pub fn new(
        cfg: Config,
        sui_client: SuiClient,
        postgres: PgConnection,
        redis: redis::Client,
    ) -> Self {
        Self {
            cfg,
            sui_client,
            postgres,
            redis,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        //todo insert to db
        let mut check_point = 0;

        loop {
            let (check_point_data, transactions, object_changed) =
                self.download_checkpoint_data(check_point).await?;

            self.collection_indexer_work(&object_changed).await?;
            self.token_indexer_work(&object_changed).await?;
            self.transaction_events_work(&transactions).await?;

            //  info!(transactions = transactions.len(),check_point, "Downloaded transactions");
            // let object_changes = transactions.iter().map(|tx| {
            //     tx.object_changes.unwrap_or_default().iter().map(|object|object.clone()).collect::<Vec<_>>()
            // }).collect::<Vec<_>>();

            // let object_changes_with_display = object_changes.iter().map(|tx|{
            //
            // }).collect::<Vec<_>>();

            //info!(object_changes?=object_changes,"Object Changes");

            check_point += 1;
        }

        Ok(())
    }

    async fn download_checkpoint_data(
        &self,
        seq: CheckpointSequenceNumber,
    ) -> Result<(
        Checkpoint,
        Vec<SuiTransactionBlockResponse>,
        Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    )> {
        let mut checkpoint = self.sui_client.read_api().get_checkpoint(seq.into()).await;

        while checkpoint.is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

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

        let object_changes = transactions
            .iter()
            .flat_map(|tx| get_object_changes(tx))
            .collect::<Vec<_>>();

        let changed_objects =
            fetch_changed_objects(self.sui_client.read_api().clone(), object_changes)
                .await
                .map_err(|e| anyhow::format_err!("fetch_changed_objects err = {e}"))?;

        Ok((checkpoint, transactions, changed_objects))
    }

    async fn collection_indexer_work(
        &mut self,
        object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    ) -> Result<()> {
        //let pg = self.postgres.clone();
        let mut con = self.redis.get_connection()?;

        let collections = object_changes
            .iter()
            .filter_map(|(status, obj, sender, timestamp)| {
                let object_type = obj.type_.as_ref().unwrap().clone().to_string();
                if object_type.contains("0x2::display::Display<") {
                    let object_type = object_type
                        .strip_prefix("0x2::display::Display<")
                        .unwrap()
                        .strip_suffix(">")
                        .unwrap()
                        .to_string();
                    let object_id = obj.object_id.to_string();

                    let _: () = con
                        .hset("collections", object_type.clone(), object_id.clone())
                        .unwrap();

                    let content = obj.content.as_ref().unwrap();
                    let kv = match content {
                        SuiParsedData::MoveObject(parseObj) => {
                            parseObj.fields.clone().to_json_value()
                        }
                        SuiParsedData::Package(_) => {
                            unreachable!("Package should not be in display")
                        }
                    };

                    let fields = &kv["fields"]["contents"];
                    let kv_set = Self::json_to_kv_map(fields);

                    //let name = kv_set.get(&"name".to_string()).unwrap_or(&"".to_string()).clone();
                    //let link = kv_set.get(&"link".to_string()).unwrap_or(&"".to_string()).clone();
                    let image_url = kv_set
                        .get(&"image_url".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    let description = kv_set
                        .get(&"description".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    //let project_url = kv_set.get(&"project_url".to_string()).unwrap_or(&"".to_string()).clone();
                    let creator = kv_set
                        .get(&"creator".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();

                    let collection = Collection {
                        chain_id: 1,
                        slug: "".to_string(),
                        collection_id: object_id,
                        collection_type: object_type,
                        creator_address: sender.clone(),
                        collection_name: creator,
                        description,
                        supply: 0,
                        version: obj.version.value() as i64,
                        metadata_uri: image_url,
                        metadata: fields.to_string(),
                        floor_sell_id: None,
                        floor_sell_value: None,
                        floor_sell_coin_id: None,
                        best_bid_id: None,
                        best_bid_value: None,
                        best_bid_coin_id: None,
                        verify: false,
                        last_metadata_sync: Some(Utc::now().naive_utc()),
                        created_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64)
                            .unwrap(),
                        updated_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64)
                            .unwrap(),
                    };
                    return Some((status, collection));
                }

                None
            })
            .collect::<Vec<(&ObjectStatus, Collection)>>();

        let insert_collections = collections
            .iter()
            .filter_map(|(objects, collection)| {
                if *objects == &ObjectStatus::Created {
                    return Some(collection.clone());
                }
                None
            })
            .collect::<Vec<Collection>>();

        batch_insert(&mut self.postgres, &insert_collections)?;
        let created_activities = insert_collections
            .iter()
            .map(|collection| {
                Activity::new_from_collection_with_type(ActivityType::Created, collection)
            })
            .collect::<Vec<Activity>>();
        batch_insert_activities(&mut self.postgres, &created_activities)?;

        // let changed_collections = collections.iter().filter_map(|objects, collection|{
        //     if objects == &ObjectStatus::Mutated || objects == &ObjectStatus::Wrapped {
        //         return Some(collection);
        //     }
        //     None
        // }).collect::<Vec<Collection>>();

        // let delete_collections = collections.iter().filter_map(|objects, collection|{
        //     if objects == &ObjectStatus::Deleted || objects == &ObjectStatus::UnwrappedThenDeleted {
        //         return Some(collection);
        //     }
        //     None
        // }).collect::<Vec<Collection>>();

        Ok(())
    }

    async fn token_indexer_work(
        &mut self,
        object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    ) -> Result<()> {
        let mut con = self.redis.get_connection()?;

        let tokens = object_changes
            .iter()
            .filter_map(|(status, obj, sender, timestamp)| {
                let object_type = obj.type_.as_ref().unwrap().clone().to_string();

                if con.hexists("collections", object_type.clone()).unwrap() {
                    let content = obj.content.as_ref().unwrap();
                    let collection_id = con.hget("collections", object_type.clone()).unwrap();

                    dbg!(content);
                    dbg!(&status);

                    let (kv, pkg) = match content {
                        SuiParsedData::MoveObject(parseObj) => (
                            parseObj.fields.clone().to_json_value(),
                            (
                                parseObj.type_.address.clone(),
                                parseObj.type_.module.clone(),
                                parseObj.type_.name.clone(),
                            ),
                        ),
                        SuiParsedData::Package(_) => {
                            unreachable!("Package should not be in display")
                        }
                    };
                    dbg!(&kv);
                    let kv_set = Self::json_to_kv_map(&kv);
                    dbg!(&kv_set);
                    let name = kv_set
                        .get(&"name".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    let image_url = kv_set
                        .get(&"image_url".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    let owner_address = obj
                        .owner
                        .as_ref()
                        .map(|owner| owner.get_owner_address().unwrap_or_default().to_string());

                    let mut collection_addr = pkg.0.to_string();
                    collection_addr.insert_str(0, &"0x");

                    return Some((
                        status,
                        (
                            Token {
                                chain_id: 1,
                                token_id: obj.object_id.to_string(),
                                collection_id,
                                creator_address: collection_addr,
                                collection_name: pkg.2.to_string(),
                                token_name: name,
                                attributes: Some(kv.to_string()),
                                version: obj.version.value() as i64,
                                payee_address: "".to_string(),
                                royalty_points_numerator: 0,
                                royalty_points_denominator: 0,
                                owner_address,
                                metadata_uri: image_url,
                                metadata_json: Some(kv.to_string()),
                                image: None,
                                created_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64)
                                    .unwrap(),
                                updated_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64)
                                    .unwrap(),
                            },
                            sender.clone(),
                        ),
                    ));
                }
                None
            })
            .collect::<Vec<(&ObjectStatus, (Token, String))>>();

        let insert_tokens = tokens
            .iter()
            .filter_map(|(objects, token)| {
                if *objects == &ObjectStatus::Created {
                    return Some(token.clone());
                }
                None
            })
            .collect::<Vec<(Token, String)>>();
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            insert_tokens.clone().into_iter().unzip();
        batch_insert_tokens(&mut self.postgres, &tokens_for_db)?;

        let mint_activitis = insert_tokens
            .iter()
            .map(|token| Activity::new_from_token_with_type(ActivityType::Minted, token))
            .collect::<Vec<Activity>>();
        batch_insert_activities(&mut self.postgres, &mint_activitis)?;

        let changed_tokens = tokens
            .iter()
            .filter_map(|(objects, token)| {
                if *objects == &ObjectStatus::Mutated || *objects == &ObjectStatus::Wrapped {
                    return Some(token.clone());
                }
                None
            })
            .collect::<Vec<(Token, String)>>();
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            changed_tokens.clone().into_iter().unzip();
        batch_change_tokens(&mut self.postgres, &tokens_for_db)?;

        let transfer_activitis = changed_tokens
            .iter()
            .map(|token| Activity::new_from_token_with_type(ActivityType::Transferred, token))
            .collect::<Vec<Activity>>();
        batch_insert_activities(&mut self.postgres, &transfer_activitis)?;

        Ok(())
    }

    async fn transaction_events_work(
        &self,
        object_changes: &Vec<SuiTransactionBlockResponse>,
    ) -> Result<()> {
        Ok(())
    }

    fn json_to_kv_map(fields: &Value) -> HashMap<String, String> {
        let mut kv_set = HashMap::new();
        if fields.is_array() {
            for v in fields.as_array().unwrap().iter() {
                let name = v["key"].to_string();
                info!("{}", name.as_str());
                info!("{:?}", &v);
                let value = v["value"].to_string();
                kv_set.insert(name, value);
            }
        } else if fields.is_object() {
            fields.as_object().unwrap().iter().for_each(|(k, v)| {
                if k == &"id" {
                    return;
                }

                kv_set.insert(k.to_string(), v.as_str().unwrap_or("").to_string());
            });
        }
        kv_set
    }
}
