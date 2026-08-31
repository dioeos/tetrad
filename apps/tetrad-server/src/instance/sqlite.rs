use sqlx::{SqlitePool, FromRow};
use async_trait::async_trait;
use uuid::Uuid;

use crate::instance::{model::Instance, repository::{InstanceRepository, RepositoryError}};


#[derive(Debug, FromRow)]
struct InstanceRow {
    id: String,
    name: String,
    setup_completed_at_ms: Option<i64>
}

impl From<InstanceRow> for Instance {
    fn from(row: InstanceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            setup_completed_at_ms: row.setup_completed_at_ms
        }
    }
}

pub(super) struct SqliteInstanceRepository {
    db: SqlitePool
}

impl SqliteInstanceRepository {
    pub(super) fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InstanceRepository for SqliteInstanceRepository {
    async fn get(
        &self
    ) -> Result<Option<Instance>, RepositoryError> {
        let row = sqlx::query_as::<_, InstanceRow>(
            r#"
            SELECT
                id,
                name,
                setup_completed_at_ms
            FROM instances
            WHERE singleton = 1
            "#,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(Instance::from))
    }

    async fn ensure_exists(
        &self,
        name: &str
    ) -> Result<Instance, RepositoryError> {
        let mut transaction = self.db.begin().await?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO instances (
                singleton,
                id,
                name,
                setup_completed_at_ms,
                created_at_ms
            )
            VALUES (1, ?, ?, NULL, ?)
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(name)
        .bind(now_ms())
        .execute(&mut *transaction)
        .await?;

        let row = sqlx::query_as::<_, InstanceRow>(
            r#"
            SELECT
                id,
                name,
                setup_completed_at_ms
            FROM instances
            WHERE singleton = 1
            "#
        )
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(row.into())
    }
}


pub fn now_ms() -> i64 {
    let milliseconds = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
    i64::try_from(milliseconds).expect("current Unix timestamp must fit in i64 milliseconds")
}
