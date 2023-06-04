use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::collections;

// "collection_name" varchar(255) NOT NULL,
// "description" varchar NOT NULL,
// "supply" int8 NOT NULL,
// "version" int8 NOT NULL,
// "metadata_uri" text NOT NULL,
// "tx" varchar(255),
// "metadata" text NOT NULL,
// "verify" bool NOT NULL DEFAULT false,
// "last_metadata_sync" BIGINT,
// "created_at" BIGINT,
// "updated_at" BIGINT

#[derive(Insertable, Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = collections)]
pub struct Collection {
    pub chain_id: i32,
    pub slug: Option<String>,
    pub collection_id: String,
    pub collection_type: String,
    pub creator_address: String,
    pub royaltie: Option<String>,
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
    pub tx: Option<String>,
    pub metadata: String,
    pub verify: bool,
    pub last_metadata_sync: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Queryable, PartialEq, Debug, Clone)]
#[diesel(table_name = tokens)]
pub struct CollectionMetadata {
    pub collection_id: String,
    pub display_name: Option<String>,
    pub collection_name: String,
    pub icon: Option<String>,
    pub description: String,
    pub supply: i64,
}

pub fn query_collection(
    connection: &mut PgConnection,
    c_id: &str,
) -> Result<Collection> {
    use crate::schema::collections::dsl::*;

    collections
        .filter(collection_id.eq(c_id))
        .limit(1)
        .get_result::<Collection>(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn update_collection_metadata(
    connection: &mut PgConnection,
    c_id: &str,
    new_meta: &Collection,
) -> Result<()> {
    use crate::schema::collections::dsl::*;

    let _ = diesel::update(collections)
        .set((
            (display_name.eq(new_meta.display_name.clone())),
            (description.eq(new_meta.description.clone())),
            (icon.eq(new_meta.icon.clone())),
            (supply.eq(new_meta.supply.clone())),
        ))
        .filter(collection_id.eq(c_id))
        .execute(connection)?;

    Ok(())
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
