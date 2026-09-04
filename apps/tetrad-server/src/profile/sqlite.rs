use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};
use torii::UserId;
use uuid::Uuid;

use crate::common::time;

use super::{
    model::{NewProfile, Profile},
    repository::{ProfileRepository, ProfileRepositoryError},
};

#[derive(Debug, FromRow)]
struct ProfileRow {
    torii_user_id: String,
    external_id: String,
}

impl From<ProfileRow> for Profile {
    fn from(row: ProfileRow) -> Self {
        Self {
            torii_user_id: UserId::new(&row.torii_user_id),
            external_id: Uuid::parse_str(&row.external_id)
                .expect("stored profile external_id must be a UUID"),
        }
    }
}

pub(super) struct SqliteProfileRepository {
    db: SqlitePool,
}

impl SqliteProfileRepository {
    pub(super) fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProfileRepository for SqliteProfileRepository {
    async fn create_profile(
        &self,
        new_profile: NewProfile,
    ) -> Result<Profile, ProfileRepositoryError> {
        let created_at_ms = time::now_ms();
        let mut transaction = self.db.begin().await?;

        let result = sqlx::query(
            r#"
            INSERT INTO user_profiles (
                torii_user_id,
                external_id,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(new_profile.torii_user_id.to_string())
        .bind(new_profile.external_id.to_string())
        .bind(created_at_ms)
        .bind(created_at_ms)
        .execute(&mut *transaction)
        .await?;

        let internal_id = result.last_insert_rowid();

        let row = sqlx::query_as::<_, ProfileRow>(
            r#"
            SELECT
                torii_user_id,
                external_id
            FROM user_profiles
            WHERE id = ?
            "#,
        )
        .bind(internal_id)
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(row.into())
    }
}
