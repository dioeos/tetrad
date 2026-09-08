use std::str::FromStr;

use sea_query::{Expr, Iden, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::FromRow;
use time::Timestamp;
use uuid::Uuid;

use crate::model::base::{CommonIden, Fields, IntoFields, prep_fields_for_create};

use super::{ModelManager, base::DbBmc, error::Error};

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
        let convert_to_timestamp =
            |ms: i64| Timestamp::from_milliseconds(ms).map_err(Error::InvalidInstanceTimestamp);

        let convert_to_uuid = |id: &str| Uuid::from_str(id).map_err(Error::InvalidInstanceUuid);

        Ok(Instance {
            id: row.id,
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

pub(in crate::model) struct InstanceForUpdate {
    pub(in crate::model) setup_completed_at_ms: Option<Timestamp>,
    pub(in crate::model) updated_at_ms: Timestamp,
}

pub(in crate::model) struct InstanceFilter {
    pub(in crate::model) name: Option<String>,
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
        instance_c: InstanceForCreate,
    ) -> Result<i64, Error> {
        let instance_fi = InstanceForInsert {
            name: instance_c.name,
            uuid: Uuid::now_v7().to_string(),
        };
        let mut fields = instance_fi.into_fields();
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
            // A skipped insert returns no row; read the existing singleton.
            Err(super::store::dbx::Error::Sqlx(sqlx::Error::RowNotFound)) => {
                let (id,) = mm
                    .dbx()
                    .fetch_one(sqlx::query_as::<_, (i64,)>(
                        "SELECT id FROM instances WHERE id = 1",
                    ))
                    .await
                    .map_err(Error::FailedToGetExistingInstance)?;

                Ok(id)
            }
            Err(error) => Err(Error::FailedToInsertInstance(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn read_instances(mm: &ModelManager) -> Vec<InstanceRow> {
        sqlx::query_as::<_, InstanceRow>(
            "SELECT id, uuid, name, setup_completed_at_ms,
                    created_at_ms, updated_at_ms
             FROM instances",
        )
        .fetch_all(mm.dbx().db())
        .await
        .expect("read instances")
    }

    #[tokio::test]
    async fn ensure_exists_creates_instance_when_missing() {
        let mm = ModelManager::new("sqlite::memory:")
            .await
            .expect("create model manager");
        let before = Timestamp::now().as_milliseconds();

        let id = InstanceBmc::ensure_exists(
            &mm,
            InstanceForCreate {
                name: "test-instance".to_owned(),
            },
        )
        .await
        .expect("create instance");

        let after = Timestamp::now().as_milliseconds();
        let rows = read_instances(&mm).await;
        assert_eq!(rows.len(), 1);

        let row = &rows[0];
        assert_eq!(id, 1);
        assert_eq!(row.id, id);
        assert_eq!(row.name, "test-instance");
        assert!(row.setup_completed_at_ms.is_none());
        let uuid = Uuid::parse_str(&row.uuid).expect("valid UUID");
        assert_eq!(uuid.get_version_num(), 7);
        assert!((before..=after).contains(&row.created_at_ms));
        assert_eq!(row.updated_at_ms, row.created_at_ms);

        mm.dbx().db().close().await;
    }

    #[tokio::test]
    async fn ensure_exists_preserves_existing_instance() {
        let mm = ModelManager::new("sqlite::memory:")
            .await
            .expect("create model manager");
        let first_id = InstanceBmc::ensure_exists(
            &mm,
            InstanceForCreate {
                name: "original-instance".to_owned(),
            },
        )
        .await
        .expect("create instance");

        sqlx::query(
            "UPDATE instances
             SET setup_completed_at_ms = 3000,
                 created_at_ms = 1000, updated_at_ms = 2000
             WHERE id = 1",
        )
        .execute(mm.dbx().db())
        .await
        .expect("set existing instance timestamps");
        let before = read_instances(&mm).await;
        assert_eq!(before.len(), 1);

        let second_id = InstanceBmc::ensure_exists(
            &mm,
            InstanceForCreate {
                name: "replacement-name".to_owned(),
            },
        )
        .await
        .expect("return existing instance");

        let after = read_instances(&mm).await;
        assert_eq!(second_id, first_id);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id, before[0].id);
        assert_eq!(after[0].uuid, before[0].uuid);
        assert_eq!(after[0].name, before[0].name);
        assert_eq!(
            after[0].setup_completed_at_ms,
            before[0].setup_completed_at_ms
        );
        assert_eq!(after[0].created_at_ms, before[0].created_at_ms);
        assert_eq!(after[0].updated_at_ms, before[0].updated_at_ms);

        mm.dbx().db().close().await;
    }

    #[tokio::test]
    async fn ensure_exists_reports_insert_failure() {
        let mm = ModelManager::new("sqlite::memory:")
            .await
            .expect("create model manager");
        mm.dbx().db().close().await;

        let result = InstanceBmc::ensure_exists(
            &mm,
            InstanceForCreate {
                name: "test-instance".to_owned(),
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(Error::FailedToInsertInstance(
                super::super::store::dbx::Error::Sqlx(sqlx::Error::PoolClosed)
            ))
        ));
    }
}
