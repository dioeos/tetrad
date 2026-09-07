use sqlx::{FromRow};
use time::Timestamp;

use super::{
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
    pub(in crate::model) id: String,
    pub(in crate::model) name: String,
    pub(in crate::model) setup_completed_at_ms: Option<Timestamp>,
    pub(in crate::model) created_at_ms: Timestamp,
    pub(in crate::model) updated_at_ms: Timestamp,
}

#[derive(Debug, FromRow)]
pub(in crate::model::instance) struct InstanceRow {
    pub(in crate::model::instance) id: String,
    pub(in crate::model::instance) name: String,
    pub(in crate::model::instance) setup_completed_at_ms: Option<i64>,
    pub(in crate::model::instance) created_at_ms: i64,
    pub(in crate::model::instance) updated_at_ms: i64,
}

impl TryFrom<InstanceRow> for Instance {
    type Error = Error;
    fn try_from(row: InstanceRow) -> Result<Instance, Self::Error> {
        let convert = |ms| {
            Timestamp::from_milliseconds(ms)
                .map_err(Error::InvalidInstanceTimestamp)
        };

        Ok(Instance {
            id: row.id,
            name: row.name,
            setup_completed_at_ms: row.setup_completed_at_ms.map(convert).transpose()?,
            created_at_ms: convert(row.created_at_ms)?,
            updated_at_ms: convert(row.updated_at_ms)?
        })
    }
}

// #[derive(Deserialize)]
pub(in crate::model) struct InstanceForCreate {
    pub(in crate::model) name: String
}

pub(in crate::model) struct InstanceForUpdate {
    pub(in crate::model) setup_completed_at_ms: Option<Timestamp>,
    pub(in crate::model) updated_at_ms: Timestamp
}


pub struct TaskBmc;

impl TaskBmc {
    pub async fn create(
        mm: &ModelManager,
        instance_c: InstanceForCreate
    ) -> Result<i64, Error> {
        let (id, ) = sqlx::query_as::<_, (i64,)>(
            "INSERT INTO instances (name) values ($1) returning id",
        )
        .bind(instance_c.name)
        .fetch_one(mm.dbx().db())
        .await?;

        Ok(id)
    }
}
