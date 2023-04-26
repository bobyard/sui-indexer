use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;

use crate::schema::lists;

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = lists)]
pub struct List {
    pub chain_id: i64,
    pub coin_id: i32,
    pub list_id: String,
    pub list_time: chrono::NaiveDateTime,
    pub token_id: String,
    pub seller_address: String,
    pub seller_value: i64,
    pub expire_time: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub fn batch_insert(connection: &mut PgConnection, records: &Vec<List>) -> Result<usize> {
    insert_into(lists::table)
        .values(records)
        .execute(connection)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
