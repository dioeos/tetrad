use std::str::FromStr;

use sea_query::{DynIden, Expr, Iden, IntoIden, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlSafeStr, sqlite::SqliteRow};
use time::Timestamp;
use uuid::Uuid;

use crate::model::{
    ModelManager,
    base::{CommonIden, DbBmc, Fields, IntoFields, crud_fns::prep_fields_for_create},
};

use super::error::Error;

#[derive(Debug, Serialize)]
pub struct Instance {
    pub uuid: Uuid,
    pub name: String,
    #[serde(serialize_with = "serialize_optional_timestamp_ms")]
    pub setup_completed_at_ms: Option<Timestamp>,
    #[serde(serialize_with = "serialize_timestamp_ms")]
    pub created_at_ms: Timestamp,
    #[serde(serialize_with = "serialize_timestamp_ms")]
    pub updated_at_ms: Timestamp,
}

fn serialize_timestamp_ms<S>(ts: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(ts.as_milliseconds())
}

fn serialize_optional_timestamp_ms<S>(
    ts: &Option<Timestamp>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match ts {
        Some(ts) => serializer.serialize_some(&ts.as_milliseconds()),
        None => serializer.serialize_none(),
    }
}

//@NOTE: This is not the DTO that the client receives. It is the view / representation
//       of the instance table, meaning it is just a subset of the possible attributes that
//       should be selected. It is possible that a field in `FromRow` struct that is not present
//       in its corresponding DTO, as that field would only be used in internal business logic
#[derive(Debug, FromRow)]
pub struct InstanceRow {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub setup_completed_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl TryFrom<InstanceRow> for Instance {
    type Error = Error;
    fn try_from(row: InstanceRow) -> Result<Instance, Self::Error> {
        let convert_to_timestamp =
            |ms: i64| Timestamp::from_milliseconds(ms).map_err(Error::InvalidInstanceTimestamp);

        let convert_to_uuid = |id: &str| Uuid::from_str(id).map_err(Error::InvalidInstanceUuid);

        Ok(Instance {
            uuid: convert_to_uuid(&row.uuid)?,
            name: row.name,
            setup_completed_at_ms: row
                .setup_completed_at_ms
                .map(convert_to_timestamp)
                .transpose()?,
            created_at_ms: convert_to_timestamp(row.created_at_ms)?,
            updated_at_ms: convert_to_timestamp(row.updated_at_ms)?,
        })
    }
}

#[derive(Iden)]
enum InstanceIden {
    Uuid,
    Name,
    SetupCompletedAtMs,
}

#[derive(Deserialize)]
pub struct InstanceForCreate {
    pub name: String,
}

pub struct InstanceForInsert {
    pub name: String,
    pub uuid: String,
}

impl IntoFields for InstanceForInsert {
    fn into_fields(self) -> Fields {
        let mut fields = Fields::default();
        fields.push_value(InstanceIden::Name, self.name);
        fields.push_value(InstanceIden::Uuid, self.uuid);
        fields
    }
}

pub struct InstanceBmc;

impl DbBmc for InstanceBmc {
    const TABLE: &'static str = "instances";
}

impl InstanceBmc {
    pub async fn ensure_exists(
        mm: &ModelManager,
        instance_c: InstanceForCreate,
    ) -> Result<i64, Error> {
        let instance_fi = InstanceForInsert {
            name: instance_c.name,
            uuid: Uuid::now_v7().to_string(),
        };
        let mut fields: Fields = instance_fi.into_fields();
        fields.push_value(CommonIden::Id, 1_i64);

        prep_fields_for_create::<Self>(&mut fields);

        let (columns, values) = fields.insert_columns_and_values();

        let mut query = Query::insert();
        query
            .into_table(Self::table_ref())
            .columns(columns)
            .values(values.into_iter().map(Expr::from))?
            .on_conflict(OnConflict::column(CommonIden::Id).do_nothing().to_owned())
            .returning(Query::returning().columns([CommonIden::Id]));

        let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(sqlx::AssertSqlSafe(sql), values);

        match mm.dbx().fetch_one(sqlx_query).await {
            Ok((id,)) => Ok(id),
            Err(super::store::dbx::error::Error::Sqlx(sqlx::Error::RowNotFound)) => {
                //@NOTE: error occurs during conflict via `do_nothing` on ID conflict.
                //       If fetch_one fails to execute, returns `RowNotFound`.
                let (id,) = mm
                    .dbx()
                    .fetch_one(sqlx::query_as::<_, (i64,)>(
                        "SELECT id FROM instances WHERE id = 1",
                    ))
                    .await
                    .map_err(Error::FailedToInsertInstance)?;

                Ok(id)
            }
            Err(err) => Err(Error::FailedToInsertInstance(err)),
        }
    }
}
