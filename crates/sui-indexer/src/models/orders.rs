use crate::schema::orders;
use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::OrderType"]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Sold,
    Offer,
    Exchange,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = orders)]
pub struct Order {
    pub chain_id: i64,
    pub coin_id: i32,
    pub list_id: String,
    pub token_id: String,
    pub offer_id: Option<String>,
    pub seller_address: String,
    pub buyer_address: String,
    pub order_type: OrderType,
    pub value: i64,
    pub sell_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub fn batch_insert(
    connection: &mut PgConnection, records: &Vec<Order>,
) -> Result<usize> {
    insert_into(orders::table)
        .values(records)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
