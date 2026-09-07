use super::{CommonIden, DbBmc, TimestampIden};
use crate::model::{Error, ModelManager};
use modql::field::{HasSeaFields, SeaField, SeaFields};
use sea_query::{Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use time::Timestamp;

pub(in crate::model) async fn create<BMC, ETY>(mm: &ModelManager, data: ETY) -> Result<i64, Error>
where
    BMC: DbBmc,
    //entity model with derived sea fields
    ETY: HasSeaFields,
{
    let mut fields = data.not_none_sea_fields();
    prep_fields_for_create::<BMC>(&mut fields);

    let (columns, sea_values) = fields.for_sea_insert();
    let mut query = Query::insert();
    query
        .into_table(BMC::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIden::Id]));

    let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
    let (id,) = mm.dbx().fetch_one(sqlx_query).await?;

    Ok(1)
}

fn prep_fields_for_create<BMC>(fields: &mut SeaFields)
where
    BMC: DbBmc,
{
    if BMC::has_timestamps() {
        add_timestamps_for_create(fields);
    }
}

fn add_timestamps_for_create(fields: &mut SeaFields) {
    let now_ms = Timestamp::now().as_milliseconds();

    fields.push(SeaField::new(TimestampIden::CreatedAtMs, now_ms));
    fields.push(SeaField::new(TimestampIden::UpdatedAtMs, now_ms));
}
