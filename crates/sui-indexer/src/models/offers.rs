use crate::schema::offers;
use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::OfferType"]
#[serde(rename_all = "snake_case")]
pub enum OfferType {
    Listed,
    Expired,
    Canceled,
    Sold,
}

#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = offers)]
pub struct Offer {
    pub chain_id: i64,
    pub coin_id: i32,
    pub offer_id: String,
    pub list_id: String,
    pub buyer_address: String,
    pub offer_value: i64,
    pub offer_type: OfferType,
    pub expire_time: chrono::NaiveDateTime,
    pub offer_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub fn batch_insert(
    connection: &mut PgConnection, records: &Vec<Offer>,
) -> Result<usize> {
    insert_into(offers::table)
        .values(records)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn delete(connection: &mut PgConnection, offer_id: &str) -> Result<usize> {
    diesel::update(offers::table.filter(offers::offer_id.eq(offer_id)))
        .set(offers::offer_type.eq(OfferType::Canceled))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
