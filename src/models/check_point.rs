use anyhow::Result;
use diesel::insert_into;
use diesel::prelude::*;

use crate::schema::check_point;
use crate::schema::check_point::dsl::*;

#[derive(Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = check_point)]
pub struct CheckPoint {
    pub chain_id: i64,
    pub version: i64,
}

pub fn query_check_point(conn: &mut PgConnection, id: i64) -> Result<i64> {
    let mut check = check_point
        .filter(chain_id.eq(id))
        .load::<(i64, i64)>(conn)?;
    if check.len() == 0 {
        let new_check_point = CheckPoint {
            chain_id: id,
            version: 0,
        };

        insert_into(check_point)
            .values(&new_check_point)
            .execute(conn)?;
        return Ok(0);
    }

    Ok(check[0].1)
}
