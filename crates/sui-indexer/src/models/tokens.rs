use crate::schema::tokens;
use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::upsert::excluded;
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = tokens)]
pub struct Token {
    pub chain_id: i64,
    pub token_id: String,
    pub collection_id: String,
    pub creator_address: String,
    pub collection_type: String,
    pub collection_name: String,
    pub token_name: String,
    pub attributes: Option<String>,
    pub version: i64,
    pub payee_address: String,
    pub royalty_points_numerator: i64,
    pub royalty_points_denominator: i64,
    pub owner_address: Option<String>,
    pub metadata_uri: String,
    pub metadata_json: Option<String>,
    pub image: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, PartialEq, Debug, Clone)]
#[diesel(table_name = tokens)]
pub struct Metadata {
    pub token_id: String,
    pub metadata_json: Option<String>,
    pub metadata_uri: String,
    pub image: Option<String>,
}

pub fn batch_insert(connection: &mut PgConnection, new_tokens: &Vec<Token>) -> Result<usize> {
    insert_into(tokens::table)
        .values(new_tokens)
        .on_conflict(tokens::token_id)
        .do_nothing()
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn batch_change(connection: &mut PgConnection, changed: &Vec<Token>) -> Result<usize> {
    insert_into(tokens::table)
        .values(changed)
        .on_conflict(tokens::token_id)
        .do_update()
        .set((
            tokens::metadata_json.eq(excluded(tokens::metadata_json)),
            tokens::version.eq(excluded(tokens::version)),
            tokens::owner_address.eq(excluded(tokens::owner_address)),
            tokens::updated_at.eq(excluded(tokens::updated_at)),
        ))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn query_the_uncache_images(connection: &mut PgConnection) -> Result<Vec<Metadata>> {
    use crate::schema::tokens::dsl::*;

    tokens
        .select((token_id, metadata_json, metadata_uri, image))
        .filter(image.is_null())
        .limit(1000)
        .get_results::<Metadata>(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn update_image_url(
    connection: &mut PgConnection,
    token_id_for_update: String,
    images_url: Option<String>,
) -> Result<()> {
    use crate::schema::tokens::dsl::*;
    let _ = diesel::update(tokens.filter(token_id.eq(token_id_for_update)))
        .set(image.eq(images_url))
        .get_result::<Token>(connection)?;

    Ok(())
}

pub fn delete(connection: &mut PgConnection, token_id: &str) -> Result<usize> {
    diesel::delete(tokens::table.filter(tokens::token_id.eq(token_id)))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
