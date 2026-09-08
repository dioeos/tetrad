use super::{CommonIden, DbBmc, Fields, IntoFields, TimestampIden};
use crate::model::{Error, ModelManager};
use sea_query::{Expr, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use time::Timestamp;

pub(in crate::model) async fn create<BMC, ETY>(mm: &ModelManager, data: ETY) -> Result<i64, Error>
where
    BMC: DbBmc,
    //entity model with derived sea fields
    ETY: IntoFields,
{
    let mut fields = data.into_fields();
    prep_fields_for_create::<BMC>(&mut fields);

    let (columns, values) = fields.insert_columns_and_values();
    let mut query = Query::insert();
    query
        .into_table(BMC::table_ref())
        .columns(columns)
        .values(values.into_iter().map(Expr::from))?
        .returning(Query::returning().columns([CommonIden::Id]));

    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(sqlx::AssertSqlSafe(sql), values);
    let (id,) = mm.dbx().fetch_one(sqlx_query).await?;

    Ok(id)
}

pub(in crate::model) fn prep_fields_for_create<BMC>(fields: &mut Fields)
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
