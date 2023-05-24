use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::collections;

#[derive(Insertable, Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct Collection {
    pub chain_id: i32,
    pub slug: String,
    pub collection_id: String,
    pub collection_type: String,
    pub creator_address: String,
    pub display_name: Option<String>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
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

#[derive(Queryable, PartialEq, Debug, Clone)]
#[diesel(table_name = tokens)]
pub struct CollectionMetadata {
    pub collection_id: String,
    pub display_name: Option<String>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub icon: Option<String>,
    pub description: String,
}

pub fn query_collection(
    connection: &mut PgConnection, c_id: &str,
) -> Result<CollectionMetadata> {
    use crate::schema::collections::dsl::*;

    collections
        .select((
            collection_id,
            display_name,
            website,
            discord,
            twitter,
            icon,
            description,
        ))
        .filter(collection_id.eq(c_id))
        .limit(1)
        .get_result::<CollectionMetadata>(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn update_collection_metadata(
    connection: &mut PgConnection, c_id: &str, new_meta: &CollectionMetadata,
) -> Result<()> {
    use crate::schema::collections::dsl::*;

    let _ = diesel::update(collections)
        .set((
            (display_name.eq(new_meta.display_name.clone())),
            (description.eq(new_meta.description.clone())),
            (icon.eq(new_meta.icon.clone())),
        ))
        .filter(collection_id.eq(c_id))
        .execute(connection)?;

    Ok(())
}

pub fn batch_insert(
    connection: &mut PgConnection, new_collections: &Vec<Collection>,
) -> Result<usize> {
    insert_into(collections::table)
        .values(new_collections)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
