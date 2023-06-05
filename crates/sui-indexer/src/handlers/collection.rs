use crate::models::activities::{Activity, ActivityType};
use crate::models::collections::Collection;
use crate::utils::json_to_kv_map;
use crate::ObjectStatus;
use anyhow::Result;
use chrono::Utc;

use redis::Commands;
use std::collections::HashMap;
use sui_sdk::rpc_types::{SuiObjectData, SuiParsedData};

pub fn parse_collection(
    object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    con: &mut redis::Connection,
    coll_set: &mut HashMap<String, String>,
) -> Result<Vec<(ObjectStatus, Collection)>> {
    Ok(object_changes
        .into_iter()
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
                coll_set.insert(object_type.clone(), object_id.clone());

                let content = obj.content.as_ref().unwrap();
                let kv = match content {
                    SuiParsedData::MoveObject(parse_obj) => {
                        parse_obj.fields.clone().to_json_value()
                    }
                    SuiParsedData::Package(_) => {
                        unreachable!("Package should not be in display")
                    }
                };

                let fields = &kv["fields"]["contents"];
                let kv_set = json_to_kv_map(fields);

                let image_url = kv_set
                    .get(&"image_url".to_string())
                    .unwrap_or(&"".to_string())
                    .clone();
                let description = kv_set
                    .get(&"description".to_string())
                    .unwrap_or(&"".to_string())
                    .clone();
                let project_url =
                    kv_set.get(&"project_url".to_string()).cloned();

                //let project_url =
                // kv_set.get(&"project_url".to_string()).unwrap_or(&"".
                // to_string()).clone();
                //let creator = kv_set.get(&"creator".to_string()).cloned();

                let collection_data_in_json =
                    serde_json::to_string(&kv_set).unwrap();
                let collection_name =
                    object_type.split("::").last().unwrap().to_string();

                let tx: Option<String> =
                    if let Some(ok) = obj.previous_transaction {
                        Some(ok.to_string())
                    } else {
                        None
                    };

                let collection = Collection {
                    chain_id: 1,
                    slug: None,
                    collection_id: object_id,
                    collection_type: object_type,
                    creator_address: sender.clone(),
                    royaltie: None,
                    display_name: None,
                    website: project_url,
                    discord: None,
                    twitter: None,
                    icon: None,
                    banner: None,
                    collection_name,
                    description,
                    supply: 0,
                    version: obj.version.value() as i64,
                    metadata_uri: image_url,
                    metadata: collection_data_in_json,
                    tx,
                    verify: false,
                    last_metadata_sync: Utc::now()
                        .naive_utc()
                        .timestamp_millis(),
                    created_at: *timestamp as i64,
                    updated_at: *timestamp as i64,
                };
                return Some((status.clone(), collection));
            }
            None
        })
        .collect::<Vec<(ObjectStatus, Collection)>>())
}

pub fn collection_indexer_work(
    collections: &Vec<(ObjectStatus, Collection)>,
) -> Result<(Vec<Collection>, Vec<Activity>)> {
    let insert_collections = collections
        .iter()
        .filter_map(|(objects, collection)| {
            if *objects == ObjectStatus::Created {
                return Some(collection.clone());
            }
            None
        })
        .collect::<Vec<Collection>>();
    // if insert_collections.is_empty() {
    //     return Ok(());
    // }

    //batch_insert(pg, &insert_collections)?;
    let created_activities = insert_collections
        .iter()
        .map(|collection| {
            Activity::new_from_collection_with_type(
                ActivityType::Created,
                collection,
            )
        })
        .collect::<Vec<Activity>>();
    //batch_insert_activities(pg, &created_activities)?;

    Ok((insert_collections, created_activities))
}
