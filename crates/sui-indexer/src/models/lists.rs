use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::lists;
use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::ListType"]
#[serde(rename_all = "snake_case")]
pub enum ListType {
    Listed,
    Expired,
    Canceled,
    Sold,
}

#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = lists)]
pub struct List {
    pub chain_id: i64,
    pub coin_id: i32,
    pub list_id: String,
    pub list_time: chrono::NaiveDateTime,
    pub token_id: String,
    pub seller_address: String,
    pub seller_value: i64,
    pub list_type: ListType,
    pub expire_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = lists)]
pub struct QueryList {
    pub id: i64,
    pub chain_id: i64,
    pub coin_id: i32,
    pub list_id: String,
    pub list_time: chrono::NaiveDateTime,
    pub token_id: String,
    pub seller_address: String,
    pub seller_value: i64,
    pub list_type: ListType,
    pub expire_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub fn batch_insert(
    connection: &mut PgConnection,
    records: &Vec<List>,
) -> Result<usize> {
    insert_into(lists::table)
        .values(records)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn delete(connection: &mut PgConnection, list_id: &str) -> Result<usize> {
    diesel::update(lists::table.filter(lists::list_id.eq(list_id)))
        .set(lists::list_type.eq(ListType::Canceled))
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
