use std::collections::{HashMap, HashSet};
use sui_sdk::SuiClient;
use sui_sdk::types::messages_checkpoint::CheckpointSequenceNumber;
use crate::config::Config;
use diesel::pg::PgConnection;
use futures::future::join_all;
use anyhow::{Error, Result};
use serde_json::Value;
use tracing::{debug,info};
use sui_sdk::rpc_types::{Checkpoint, SuiObjectData, SuiParsedData, SuiTransactionBlockResponse};
use sui_sdk::types::object::Object;
use crate::{fetch_changed_objects, get_object_changes, multi_get_full_transactions, ObjectStatus};
use crate::models::collections::{batch_insert, Collection};

use crate::MULTI_GET_CHUNK_SIZE;

pub(crate) struct Indexer {
    cfg:Config,
    sui_client:SuiClient,
    postgres: PgConnection
}

impl Indexer{
    pub fn new(cfg:Config,sui_client:SuiClient,postgres:PgConnection) -> Self {
        Self {
            cfg,
            sui_client,
            postgres
        }
    }


    pub async fn start(&self) -> anyhow::Result<()>{
        //todo insert to db
        let mut check_point = 0;

        loop {
            let (check_point_data,transactions,object_changed,) = self.download_checkpoint_data(check_point).await?;

            self.collection_indexer_work(&object_changed).await?;
            self.token_indexer_work(&object_changed).await?;
            self.transaction_events_work(&transactions).await?;

            //info!(transactions = transactions.len(),check_point, "Downloaded transactions");

            // let object_changes = transactions.iter().map(|tx| {
            //     tx.object_changes.unwrap_or_default().iter().map(|object|object.clone()).collect::<Vec<_>>()
            // }).collect::<Vec<_>>();

            // let object_changes_with_display = object_changes.iter().map(|tx|{
            //
            // }).collect::<Vec<_>>();

            //info!(object_changes?=object_changes,"Object Changes");

            check_point+=1;
        }

        Ok(())
    }

    async fn download_checkpoint_data(
        &self,
        seq: CheckpointSequenceNumber,
    ) -> Result<(Checkpoint,Vec<SuiTransactionBlockResponse>,Vec<(ObjectStatus,SuiObjectData)>)> {
        let mut checkpoint = self
            .sui_client
            .read_api()
            .get_checkpoint(seq.into())
            .await;

        while checkpoint.is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            checkpoint = self
                .sui_client.read_api()
                .get_checkpoint(seq.into())
                .await;
        }

        // unwrap here is safe because we checked for error above
        let checkpoint = checkpoint.unwrap();

        let transactions = join_all(checkpoint.transactions.chunks(MULTI_GET_CHUNK_SIZE).map(
            |digests| multi_get_full_transactions(self.sui_client.read_api().clone(), digests.to_vec()),
        ))
            .await
            .into_iter()
            .try_fold(vec![], |mut acc, chunk| {
                acc.extend(chunk?);
                Ok::<Vec<SuiTransactionBlockResponse>, Error>(acc)
            })?;

        let object_changes = transactions
            .iter()
            .flat_map(|tx| get_object_changes(&tx.effects))
            .collect::<Vec<_>>();

        let changed_objects =
            fetch_changed_objects(self.sui_client.read_api().clone(), object_changes).await.map_err(|e|anyhow::format_err!("fetch_changed_objects err = {e}"))?;

        Ok((checkpoint,transactions,changed_objects))
    }

    async fn collection_indexer_work(&self, object_changes: &Vec<(ObjectStatus, SuiObjectData)>) -> Result<()>{
        let pg = self.postgres.clone();

        let _ = object_changes.iter().map(|(status,obj)| {
            let object_type = obj.type_.as_ref().unwrap().clone().to_string();
            if object_type.contains("0x2::display::Display<") {
                let object_type = object_type.strip_prefix("0x2::display::Display<").unwrap().strip_suffix(">").unwrap().to_string();
                let object_id = obj.object_id.to_string();
                let content = obj.content.as_ref().unwrap();
                let kv = match content {
                    SuiParsedData::MoveObject(parseObj) => {
                        parseObj.fields.clone().to_json_value()
                    }
                    SuiParsedData::Package(_) => {unreachable!("Package should not be in display")}
                };

                let fields = &kv["fields"]["contents"];
                let kv_set = Self::json_to_kv_map(fields);

                let name = kv_set.get(&"name".to_string()).unwrap_or(&"".to_string()).clone();
                let link = kv_set.get(&"link".to_string()).unwrap_or(&"".to_string()).clone();
                let image_url = kv_set.get(&"image_url".to_string()).unwrap_or(&"".to_string()).clone();
                let description = kv_set.get(&"description".to_string()).unwrap_or(&"".to_string()).clone();
                let project_url = kv_set.get(&"project_url".to_string()).unwrap_or(&"".to_string()).clone();
                let creator = kv_set.get(&"creator".to_string()).unwrap_or(&"".to_string()).clone();
                use chrono::prelude::*;

                let collection = Collection{
                    chain_id: 1,
                    slug: "".to_string(),
                    collection_id: object_id,
                    collection_type: object_type,
                    creator_address: "".to_string(),
                    collection_name: creator,
                    description,
                    supply: 0,
                    version: 0,
                    metadata_uri: image_url,
                    metadata: fields.to_string(),
                    floor_sell_id: None,
                    floor_sell_value: None,
                    floor_sell_coin_id: None,
                    best_bid_id: None,
                    best_bid_value: None,
                    best_bid_coin_id: None,
                    verify: false,
                    last_metadata_sync: None,
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                };

                batch_insert(&mut pg,&vec![collection]).unwrap();
                //info!("NFT {} {} {} {:?}",object_type,object_id,name,status);
            }

            ()
        }).collect::<Vec<()>>();

        Ok(())
    }

    async fn token_indexer_work(&self, object_changes: &Vec<(ObjectStatus, SuiObjectData)>) -> Result<()>{
        // let _ = object_changes.iter().map(|(status,obj)| {
        //     let object_type = obj.type_.as_ref().unwrap().clone().to_string();
        //     if object_type.contains("0x2::display::Display<") {
        //         let object_type = object_type.strip_prefix("0x2::display::Display<").unwrap().strip_suffix(">").unwrap().to_string();
        //         let object_id = obj.object_id.to_string();
        //         let contect = obj.content.as_ref().unwrap().to_string();
        //         info!("NFT {} {} {} {:?}",object_type,object_id,contect,status);
        //     }
        //
        //     ()
        // }).collect::<Vec<()>>();

        Ok(())
    }

    async fn transaction_events_work(&self,object_changes:&Vec<SuiTransactionBlockResponse>) -> Result<()>{

        Ok(())
    }


    fn json_to_kv_map(fields:&Value) -> HashMap<String,String>{
        let mut kv_set = HashMap::new();
        if fields.is_array() {
            for v in fields.as_array().unwrap().iter() {
                let name = v["key"].to_string();
                info!("{}",name.as_str());
                info!("{:?}",&v);
                let value = v["value"].to_string();
                kv_set.insert(name,value);
            }
        }
        kv_set
    }

}

