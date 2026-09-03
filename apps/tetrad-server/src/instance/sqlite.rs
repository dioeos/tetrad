use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::common::time;

use super::{
    model::Instance,
    repository::{InstanceRepository, RepositoryError},
};

#[derive(Debug, FromRow)]
struct InstanceRow {
    id: String,
    name: String,
    setup_completed_at_ms: Option<i64>,
}

impl From<InstanceRow> for Instance {
    fn from(row: InstanceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            setup_completed_at_ms: row.setup_completed_at_ms,
        }
    }
}

pub(super) struct SqliteInstanceRepository {
    db: SqlitePool,
}

impl SqliteInstanceRepository {
    pub(super) fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InstanceRepository for SqliteInstanceRepository {
    async fn get(&self) -> Result<Option<Instance>, RepositoryError> {
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

    async fn ensure_exists(&self, name: &str) -> Result<Instance, RepositoryError> {
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
        .bind(time::now_ms())
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
            "#,
        )
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(row.into())
    }
}

#[cfg(test)]
mod tests {
    use super::SqliteInstanceRepository;
    use crate::instance::{InstanceRepository, repository::RepositoryError};
    use sqlx::SqlitePool;

    #[sqlx::test(migrations = "./migrations")]
    async fn get_returns_none_when_no_instance_exists(pool: SqlitePool) {
        let repo = SqliteInstanceRepository::new(pool);
        let result = repo.get().await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_exists_creates_instance(pool: SqlitePool) {
        let repo = SqliteInstanceRepository::new(pool.clone());
        let instance = repo.ensure_exists("test-instance").await.unwrap();

        assert_eq!(instance.name, "test-instance");
        assert!(instance.setup_completed_at_ms.is_none());
        assert!(!instance.id.is_empty());

        let row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row_count, 1)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_exists_is_idempotent(pool: SqlitePool) {
        let repo = SqliteInstanceRepository::new(pool.clone());

        let first = repo.ensure_exists("first").await.unwrap();
        let second = repo.ensure_exists("second").await.unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(second.name, first.name);

        let row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row_count, 1)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_returns_ensured_instance(pool: SqlitePool) {
        let repo = SqliteInstanceRepository::new(pool);
        let ensured = repo.ensure_exists("tetrad").await.unwrap();
        let result = repo.get().await.unwrap().unwrap();

        assert_eq!(ensured.id, result.id);
        assert_eq!(ensured.name, result.name);
        assert_eq!(ensured.setup_completed_at_ms, result.setup_completed_at_ms)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn database_errors_are_wrapped_as_repository_errors(pool: SqlitePool) {
        sqlx::query("DROP TABLE instances")
            .execute(&pool)
            .await
            .unwrap();

        let repo = SqliteInstanceRepository::new(pool);
        let result = repo.get().await;
        let ensured = repo.ensure_exists("tetrad").await;

        assert!(matches!(result, Err(RepositoryError::Database(_))));
        assert!(matches!(ensured, Err(RepositoryError::Database(_))));
    }
}
