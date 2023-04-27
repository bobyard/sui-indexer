use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;

use crate::schema::offers;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = offers)]
pub struct Offer {
    pub chain_id: i64,
    pub coin_id: i32,
    pub offer_id: String,
    pub list_id: String,
    pub buyer_address: String,
    pub offer_value: i64,
    pub expire_time: chrono::NaiveDateTime,
    pub offer_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub fn batch_insert(connection: &mut PgConnection, records: &Vec<Offer>) -> Result<usize> {
    insert_into(offers::table)
        .values(records)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn delete(connection: &mut PgConnection, offer_id: &str) -> Result<usize> {
    diesel::delete(offers::table.filter(offers::offer_id.eq(offer_id)))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
