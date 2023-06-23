use crate::schema::tokens;
use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::TokenStatus"]
#[serde(rename_all = "snake_case")]
pub enum TokenStatus {
    EXIST,
    DELETE,
}

#[derive(
    Insertable, Queryable, PartialEq, Debug, Clone, Serialize, Deserialize,
)]
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
    pub tx: Option<String>,
    pub status: TokenStatus,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Queryable, PartialEq, Debug, Clone)]
#[diesel(table_name = tokens)]
pub struct Metadata {
    pub token_id: String,
    pub metadata_json: Option<String>,
    pub metadata_uri: String,
    pub image: Option<String>,
}

pub fn batch_insert(
    connection: &mut PgConnection,
    new_tokens: &Vec<Token>,
) -> Result<usize> {
    insert_into(tokens::table)
        .values(new_tokens)
        .on_conflict(tokens::token_id)
        .do_nothing()
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn batch_change(
    connection: &mut PgConnection,
    changed: &Vec<Token>,
) -> Result<usize> {
    insert_into(tokens::table)
        .values(changed)
        .on_conflict(tokens::token_id)
        .do_update()
        .set((
            tokens::metadata_json.eq(excluded(tokens::metadata_json)),
            tokens::version.eq(excluded(tokens::version)),
            tokens::owner_address.eq(excluded(tokens::owner_address)),
            tokens::updated_at.eq(excluded(tokens::updated_at)),
            tokens::tx.eq(excluded(tokens::tx)),
            //tokens::image.eq(excluded(tokens::image)),
        ))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn query_the_uncache_images(
    connection: &mut PgConnection,
) -> Result<Vec<Metadata>> {
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
        .execute(connection)?;

    Ok(())
}

/* pub fn count_star(conn: &mut PgConnection, c_id: String) -> Result<i64> {
    use crate::schema::tokens::dsl::*;
    use diesel::dsl::count_star;

    let a = tokens
        .select(count_star())
        .filter(collection_id.eq(c_id))
        .filter(status.eq(TokenStatus::EXIST))
        .first::<i64>(conn)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(a)
} */

pub fn set_status_delete(
    connection: &mut PgConnection,
    t_id: &str,
) -> Result<()> {
    use crate::schema::tokens::dsl::*;

    tracing::info!(id = t_id, "Set status_delete",);

    let _ = diesel::update(tokens.filter(token_id.eq(t_id)))
        .set(status.eq(TokenStatus::DELETE))
        .execute(connection)?;
    Ok(())
}

pub fn delete(connection: &mut PgConnection, token_id: &str) -> Result<usize> {
    diesel::delete(tokens::table.filter(tokens::token_id.eq(token_id)))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
