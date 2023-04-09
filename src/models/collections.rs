use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;

use crate::schema::collections;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = collections)]
pub struct Collection {
    pub chain_id: i32,
    pub slug: String,
    pub collection_id: String,
    pub collection_type: String,
    pub creator_address: String,
    pub collection_name: String,
    pub description: String,
    pub supply: i64,
    pub version: i64,
    pub metadata_uri: String,
    pub metadata: String,
    pub floor_sell_id: Option<i32>,
    pub floor_sell_value: Option<i64>,
    pub floor_sell_coin_id: Option<i32>,
    pub best_bid_id: Option<i32>,
    pub best_bid_value: Option<i64>,
    pub best_bid_coin_id: Option<i32>,
    pub verify: bool,
    pub last_metadata_sync: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

pub fn batch_insert(
    connection: &mut PgConnection,
    new_collections: &Vec<Collection>,
) -> Result<usize> {
    insert_into(collections::table)
        .values(new_collections)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
