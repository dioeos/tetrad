use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use time::Timestamp;

use crate::model::{
    Error, ModelManager,
    base::{CommonIden, SelectFields},
    store::dbx::SqliteRowType,
};

use super::{DbBmc, Fields, TimestampIden};

//@NOTE: `get` operates under the assumption that the entity exists.
//        To retrieve an entity that may or may not exist without
//        receiving an error, use `list` or `first`, which retrieves
//        the first in a list, given specific 'n' filters
pub async fn get<BMC, ETY>(mm: &ModelManager, id: i64) -> Result<ETY, Error>
where
    BMC: DbBmc,
    ETY: SqliteRowType + SelectFields,
{
    let mut query = Query::select();
    query
        .from(BMC::table_ref())
        .columns(ETY::select_columns())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, ETY, _>(sqlx::AssertSqlSafe(sql), values);

    let entity = mm
        .dbx()
        .fetch_optional(sqlx_query)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: BMC::TABLE,
            id,
        })?;

    Ok(entity)
}
pub fn prep_fields_for_create<BMC>(fields: &mut Fields)
where
    BMC: DbBmc,
{
    if BMC::has_timestamps() {
        add_timestamps_for_create(fields);
    }
}

fn add_timestamps_for_create(fields: &mut Fields) {
    let now_ms = Timestamp::now().as_milliseconds();
    fields.push_value(TimestampIden::CreatedAtMs, now_ms);
    fields.push_value(TimestampIden::UpdatedAtMs, now_ms);
}
