use crate::models::activities::{batch_insert as batch_insert_activities, Activity, ActivityType};
use crate::models::tokens::{
    batch_change as batch_change_tokens, batch_insert as batch_insert_tokens, Token,
};
use crate::ObjectStatus;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use diesel::PgConnection;
use redis::Commands;
use sui_sdk::rpc_types::SuiObjectData;

pub fn parse_tokens(
    object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    con: &mut redis::Connection,
) -> Result<Vec<(ObjectStatus, (Token, String))>> {
    let tokens = object_changes
        .into_iter()
        .filter_map(|(status, obj, sender, timestamp)| {
            let object_type = obj.type_.as_ref().unwrap().clone().to_string();
            if let Some(display) = &obj.display {
                if let Some(kv_set) = &display.data {
                    dbg!(kv_set);
                    let collection_id = con.hget("collections", object_type.clone()).unwrap();

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

                    let display_json = serde_json::to_string(&kv_set).unwrap();

                    return Some((
                        status.clone(),
                        (
                            Token {
                                chain_id: 1,
                                token_id: obj.object_id.to_string(),
                                collection_id,
                                collection_type: object_type.clone(),
                                creator_address: "".to_string(),
                                collection_name: "".to_string(),
                                token_name: name,
                                attributes: Some(display_json.clone()),
                                version: obj.version.value() as i64,
                                payee_address: "".to_string(),
                                royalty_points_numerator: 0,
                                royalty_points_denominator: 0,
                                owner_address,
                                metadata_uri: image_url,
                                metadata_json: Some(display_json),
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
                return None;
            }
            return None;
        })
        .collect::<Vec<(ObjectStatus, (Token, String))>>();

    Ok(tokens)
}

pub fn token_indexer_work(
    tokens: &Vec<(ObjectStatus, (Token, String))>,
    pg: &mut PgConnection,
) -> Result<()> {
    let insert_tokens = tokens
        .iter()
        .filter_map(|(objects, token)| {
            if *objects == ObjectStatus::Created {
                return Some(token.clone());
            }
            None
        })
        .collect::<Vec<(Token, String)>>();
    if insert_tokens.len() > 0 {
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            insert_tokens.clone().into_iter().unzip();

        batch_insert_tokens(pg, &tokens_for_db)
            .map_err(|e| anyhow!("BatchInsertTokens Failed {}", e.to_string()))?;
        let mint_activitis = insert_tokens
            .iter()
            .map(|token| Activity::new_from_token_with_type(ActivityType::Minted, token))
            .collect::<Vec<Activity>>();

        batch_insert_activities(pg, &mint_activitis)
            .map_err(|e| anyhow!("BatchInsertActivities Failed {}", e.to_string()))?;
    }

    let changed_tokens = tokens
        .iter()
        .filter_map(|(objects, token)| {
            if *objects == ObjectStatus::Mutated {
                return Some(token.clone());
            }
            None
        })
        .collect::<Vec<(Token, String)>>();
    if changed_tokens.len() > 0 {
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            changed_tokens.clone().into_iter().unzip();

        let tokens_for_db1 = tokens_for_db.clone();
        let tokens_for_db = tokens_for_db
            .into_iter()
            .filter(|e| {
                let mut inner_version = e.version;

                for t in &tokens_for_db1 {
                    if e.token_id == t.token_id {
                        if e.version == t.version {
                        } else {
                            inner_version = t.version;
                        }
                    }
                }

                if e.version != inner_version {
                    return false;
                }
                return true;
            })
            .collect::<Vec<Token>>();

        batch_change_tokens(pg, &tokens_for_db)
            .map_err(|e| anyhow!("BatchChangeTokens failed {}", e.to_string()))?;
    }

    Ok(())
}
