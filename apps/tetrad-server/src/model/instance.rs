use std::str::FromStr;

use sea_query::{Expr, Iden, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{FromRow};
use time::Timestamp;
use uuid::Uuid;

use crate::model::base::{CommonIden, Fields, IntoFields, prep_fields_for_create};

use super::{
    base::DbBmc,
    error::Error,
    ModelManager
};


//@NOTE: This is not the DTO that the client receives. It is
//       the view /representation of the instance table, meaning it is just a
//       subset of the possible attributes that should be selected.
//       It is possible that a field in `FromRow` struct is not present
//       in its corresponding DTO, as that field would only be 
//       used in internal business logic
#[derive(Debug)]
pub(in crate::model) struct Instance {
    pub(in crate::model) id: i64,
    pub(in crate::model) uuid: Uuid,
    pub(in crate::model) name: String,
    pub(in crate::model) setup_completed_at_ms: Option<Timestamp>,
    pub(in crate::model) created_at_ms: Timestamp,
    pub(in crate::model) updated_at_ms: Timestamp,
}

#[derive(Debug, FromRow)]
pub(in crate::model::instance) struct InstanceRow {
    pub(in crate::model::instance) id: i64,
    pub(in crate::model::instance) uuid: String,
    pub(in crate::model::instance) name: String,
    pub(in crate::model::instance) setup_completed_at_ms: Option<i64>,
    pub(in crate::model::instance) created_at_ms: i64,
    pub(in crate::model::instance) updated_at_ms: i64,
}

impl TryFrom<InstanceRow> for Instance {
    type Error = Error;
    fn try_from(row: InstanceRow) -> Result<Instance, Self::Error> {
        let convert_to_timestamp = |ms: i64| {
            Timestamp::from_milliseconds(ms)
                .map_err(Error::InvalidInstanceTimestamp)
        };

        let convert_to_uuid = |id: &str| {
            Uuid::from_str(id)
                .map_err(Error::InvalidInstanceUuid)
        };

        Ok(Instance {
            id: row.id,
            uuid: convert_to_uuid(&row.uuid)?,
            name: row.name,
            setup_completed_at_ms: row.setup_completed_at_ms.map(convert_to_timestamp).transpose()?,
            created_at_ms: convert_to_timestamp(row.created_at_ms)?,
            updated_at_ms: convert_to_timestamp(row.updated_at_ms)?
        })
    }
}

#[derive(Deserialize)]
pub struct InstanceForCreate {
    pub name: String
}

pub struct InstanceForInsert {
    pub name: String,
    pub uuid: String
}

impl IntoFields for InstanceForInsert {
    fn into_fields(self) -> Fields {
        let mut fields = Fields::default();
        fields.push_value(InstanceIden::Name, self.name);
        fields.push_value(InstanceIden::Uuid, self.uuid);
        fields
    }
}

pub(in crate::model) struct InstanceForUpdate {
    pub(in crate::model) setup_completed_at_ms: Option<Timestamp>,
    pub(in crate::model) updated_at_ms: Timestamp
}

pub(in crate::model) struct InstanceFilter {
    pub(in crate::model) name: Option<String>
}

#[derive(Iden)]
enum InstanceIden {
    Uuid,
    Name,
}

pub struct InstanceBmc;

impl DbBmc for InstanceBmc {
    const TABLE: &'static str = "instances";
}

impl InstanceBmc {
    pub async fn ensure_exists(
        mm: &ModelManager,
        instance_c: InstanceForCreate
    ) -> Result<i64, Error> {
        let instance_fi = InstanceForInsert { 
            name: instance_c.name,
            uuid: Uuid::now_v7().to_string()
        };
        let mut fields = instance_fi.into_fields();
        prep_fields_for_create::<Self>(&mut fields);

        let (columns, values) = fields.insert_columns_and_values();
        let mut query = Query::insert();
        query
            .into_table(Self::table_ref())
            .columns(columns)
            .values(values.into_iter().map(Expr::from))?
            .on_conflict(
                OnConflict::column(CommonIden::Id)
                    .do_nothing()
                    .to_owned()
            )
            .returning(Query::returning().columns([CommonIden::Id]));

        let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
        let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(sqlx::AssertSqlSafe(sql), values);
        let (id,) = mm.dbx().fetch_one(sqlx_query).await?;

        Ok(id)
    }
}
