use crate::models::activities::{Activity, ActivityType};
use crate::models::tokens::{Token, TokenStatus};
use crate::ObjectStatus;
use anyhow::Result;

use std::collections::HashMap;
use sui_sdk::rpc_types::SuiObjectData;

pub fn parse_tokens(
    object_changes: &Vec<(ObjectStatus, SuiObjectData, String, u64)>,
    coll_set: &mut HashMap<String, String>,
) -> Result<Vec<(ObjectStatus, (Token, String))>> {
    let tokens = object_changes
        .into_iter()
        .filter_map(|(status, obj, sender, timestamp)| {
            let object_type = obj.type_.as_ref().unwrap().clone().to_string();
            if let Some(display) = &obj.display {
                if let Some(kv_set) = &display.data {
                    let collection_id =
                        coll_set.get(&object_type).expect(&*format!(
                            "collection not found in kv store {}",
                            object_type
                        ));

                    let name = kv_set
                        .get(&"name".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    let image_url = kv_set
                        .get(&"image_url".to_string())
                        .unwrap_or(&"".to_string())
                        .clone();
                    let owner_address = obj.owner.as_ref().map(|owner| {
                        owner
                            .get_owner_address()
                            .unwrap_or_default()
                            .to_string()
                    });

                    let display_json = serde_json::to_string(&kv_set).unwrap();

                    let tx: Option<String> =
                        if let Some(ok) = obj.previous_transaction {
                            Some(ok.to_string())
                        } else {
                            None
                        };

                    return Some((
                        status.clone(),
                        (
                            Token {
                                chain_id: 1,
                                token_id: obj.object_id.to_string(),
                                collection_id: collection_id.to_string(),
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
                                tx,
                                status: TokenStatus::EXIST,
                                created_at: Some(*timestamp as i64),
                                updated_at: Some(*timestamp as i64),
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
) -> Result<(Vec<Token>, Vec<Activity>)> {
    let insert_tokens = tokens
        .iter()
        .filter_map(|(objects, token)| {
            if *objects == ObjectStatus::Created {
                return Some(token.clone());
            }
            None
        })
        .collect::<Vec<(Token, String)>>();
    let mut ret_tokens = vec![];
    let mut ret_act = vec![];

    if insert_tokens.len() > 0 {
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            insert_tokens.clone().into_iter().unzip();
        ret_tokens.extend_from_slice(&tokens_for_db);
        // batch_insert_tokens(pg, &tokens_for_db).map_err(|e| {
        //     anyhow!("BatchInsertTokens Failed {}", e.to_string())
        // })?;
        let mint_activitis = insert_tokens
            .iter()
            .map(|token| {
                Activity::new_from_token_with_type(ActivityType::Minted, token)
            })
            .collect::<Vec<Activity>>();

        ret_act.extend_from_slice(&mint_activitis);
        // batch_insert_activities(pg, &mint_activitis).map_err(|e| {
        //     anyhow!("BatchInsertActivities Failed {}", e.to_string())
        // })?;
    }

    let changed_tokens = tokens
        .iter()
        .filter_map(|(objects, token)| {
            if *objects == ObjectStatus::Mutated
                || *objects == ObjectStatus::Unwrapped
            {
                return Some(token.clone());
            }
            None
        })
        .collect::<Vec<(Token, String)>>();

    if changed_tokens.len() > 0 {
        let (tokens_for_db, _): (Vec<Token>, Vec<String>) =
            changed_tokens.clone().into_iter().unzip();
        // let tokens_for_db1 = tokens_for_db.clone();

        // let tokens_for_db = tokens_for_db
        //     .clone()
        //     .into_iter()
        //     .filter(|e| {
        //         let mut inner_version = e.version;

        //         for t in &tokens_for_db1 {
        //             if e.token_id == t.token_id {
        //                 if e.version == t.version {
        //                 } else {
        //                     inner_version = t.version;
        //                 }
        //             }
        //         }

        //         if e.version != inner_version {
        //             return false;
        //         }
        //         return true;
        //     })
        //     .collect::<Vec<Token>>();
        ret_tokens.extend_from_slice(&tokens_for_db);

        // let mint_activitis = insert_tokens
        //     .iter()
        //     .map(|token| {
        //         Activity::new_from_token_with_type(ActivityType::Minted,
        // token)     })
        //     .collect::<Vec<Activity>>();

        // batch_change_tokens(pg, &tokens_for_db).map_err(|e| {
        //     anyhow!("BatchChangeTokens failed {}", e.to_string())
        // })?;
    }

    if ret_tokens.len() > 0 {
        let tokens_for_db = ret_tokens
            .clone()
            .into_iter()
            .filter(|e| {
                let mut inner_version = e.version;

                for t in &ret_tokens {
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

        ret_tokens = tokens_for_db;
    }

    // let deleted_tokens = tokens
    //     .iter()
    //     .filter_map(|(objects, token)| {
    //         if *objects == ObjectStatus::Deleted
    //             || *objects == ObjectStatus::UnwrappedThenDeleted
    //         {
    //             return Some(token.clone());
    //         }
    //         None
    //     })
    //     .collect::<Vec<(Token, String)>>();
    // if deleted_tokens.len() > 0 {
    //     for t in deleted_tokens {
    //         set_status_delete(pg, &t.0.token_id).map_err(|e| {
    //             anyhow!("DeleteTokens failed {}", e.to_string())
    //         })?;
    //     }
    // }

    Ok((ret_tokens, ret_act))
}
