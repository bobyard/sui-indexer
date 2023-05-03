use crate::models::activities::{batch_insert as batch_insert_activities, Activity, ActivityType};
use crate::models::collections::{batch_insert, Collection};
use crate::utils::json_to_kv_map;
use crate::ObjectStatus;
use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use diesel::PgConnection;
use redis::Commands;
use sui_sdk::rpc_types::{SuiObjectData, SuiParsedData};

pub fn parse_collection(
    object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    con: &mut redis::Connection,
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
                let project_url = kv_set.get(&"project_url".to_string()).cloned();

                //let project_url = kv_set.get(&"project_url".to_string()).unwrap_or(&"".to_string()).clone();
                let creator = kv_set.get(&"creator".to_string()).cloned();

                let collection_data_in_json = serde_json::to_string(&kv_set).unwrap();
                let collection_name = object_type.split("::").last().unwrap().to_string();

                let collection = Collection {
                    chain_id: 1,
                    slug: "".to_string(),
                    collection_id: object_id,
                    collection_type: object_type,
                    creator_address: sender.clone(),
                    display_name: creator,
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
                    floor_sell_id: None,
                    floor_sell_value: None,
                    floor_sell_coin_id: None,
                    best_bid_id: None,
                    best_bid_value: None,
                    best_bid_coin_id: None,
                    verify: false,
                    last_metadata_sync: Some(Utc::now().naive_utc()),
                    created_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64).unwrap(),
                    updated_at: NaiveDateTime::from_timestamp_millis(*timestamp as i64).unwrap(),
                };
                return Some((status.clone(), collection));
            }
            None
        })
        .collect::<Vec<(ObjectStatus, Collection)>>())
}

pub fn collection_indexer_work(
    collections: &Vec<(ObjectStatus, Collection)>,
    pg: &mut PgConnection,
) -> Result<()> {
    let insert_collections = collections
        .iter()
        .filter_map(|(objects, collection)| {
            if *objects == ObjectStatus::Created {
                return Some(collection.clone());
            }
            None
        })
        .collect::<Vec<Collection>>();

    batch_insert(pg, &insert_collections)?;
    let created_activities = insert_collections
        .iter()
        .map(|collection| {
            Activity::new_from_collection_with_type(ActivityType::Created, collection)
        })
        .collect::<Vec<Activity>>();
    batch_insert_activities(pg, &created_activities)?;

    Ok(())
}
