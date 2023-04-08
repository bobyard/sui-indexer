use diesel::insert_into;
use diesel::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use diesel_derive_enum::DbEnum;
use crate::models::collections::Collection;
use crate::models::tokens::Token;
use crate::schema::activities;



#[derive(DbEnum, Debug, Clone, Copy, Deserialize, Serialize,PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::ActivityType"]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Created, ///only for collections
    Minted,
    Transferred,
    Listed,
    Canceled,
    Sold,
}

#[derive(Insertable,Debug,Clone)]
#[diesel(table_name = activities)]
pub struct Activity {
    pub chain_id:i64,
    pub version:i64,
    pub event_account_address: String,
    pub event_creation_number: i64,
    pub event_sequence_number: i64,
    pub collection_data_id_hash: String,
    pub token_data_id_hash: String,
    pub property_version: i64,
    pub creator_address: String,
    pub collection_name: String,
    pub name: String,
    pub transfer_type: ActivityType,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_amount: i64,
    pub coin_type: Option<String>,
    pub coin_amount: i64,
    pub transaction_timestamp: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub fn batch_insert(connection: &mut PgConnection, new: &Vec<Activity>) -> Result<usize>{
    insert_into(activities::table)
        .values(new)
        .execute(connection).map_err(|e| anyhow::anyhow!(e.to_string()))
}

impl Activity {
    pub fn new_from_collection_with_type(t:ActivityType, collection:&Collection) -> Activity {
        Activity {
            chain_id: collection.chain_id as i64,
            version: collection.version,
            event_account_address: collection.creator_address.clone(),
            event_creation_number: 0,
            event_sequence_number: 0,
            collection_data_id_hash: collection.collection_id.clone(),
            token_data_id_hash: "".to_string(),
            property_version: collection.version,
            creator_address: "".to_string(),
            collection_name: collection.collection_name.clone(),
            name: "".to_string(),
            transfer_type: t,
            from_address: None,
            to_address: None,
            token_amount: 0,
            coin_type: None,
            coin_amount: 0,
            transaction_timestamp: chrono::Utc::now().naive_utc(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn new_from_token_with_type(t:ActivityType,token:&Token) -> Activity {
        Activity {
            chain_id: token.chain_id,
            version: token.version,
            event_account_address: token.creator_address.clone(),
            event_creation_number: 0,
            event_sequence_number: 0,
            collection_data_id_hash: token.collection_id.clone(),
            token_data_id_hash: token.token_id.clone(),
            property_version: token.version,
            creator_address: "".to_string(),
            collection_name: token.collection_name.clone(),
            name: token.token_name.clone(),
            transfer_type: t,
            from_address: None,
            to_address: token.owner_address.clone(),
            token_amount: 0,
            coin_type: None,
            coin_amount: 0,
            transaction_timestamp: chrono::Utc::now().naive_utc(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

