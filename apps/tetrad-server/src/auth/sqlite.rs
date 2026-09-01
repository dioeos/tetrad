use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::common::time;

use super::{
    model::{NewUser, User},
    repository::{AuthRepository, AuthRepositoryError},
};

#[derive(Debug, FromRow)]
struct UserRow {
    internal_id: i64,
    external_id: Uuid,
    username: String,
    normalized_username: String,
    password_hash: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            internal_id: row.internal_id,
            external_id: row.external_id,
            username: row.username,
            normalized_username: row.normalized_username,
            password_hash: row.password_hash,
        }
    }
}

pub(super) struct SqliteAuthRepository {
    db: SqlitePool,
}

impl SqliteAuthRepository {
    pub(super) fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthRepository for SqliteAuthRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, AuthRepositoryError> {
        let mut transaction = self.db.begin().await?;

        let created_at_ms = time::now_ms();
        let result = sqlx::query(
            r#"
            INSERT INTO users (
                external_id,
                username,
                normalized_username,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&new_user.external_id.to_string())
        .bind(&new_user.username)
        .bind(&new_user.normalized_username)
        .bind(created_at_ms)
        .bind(created_at_ms)
        .execute(&mut *transaction)
        .await
        .map_err(map_insert_user_error)?;

        let internal_id = result.last_insert_rowid();

        sqlx::query(
            r#"
            INSERT INTO password_credentials (
                user_id,
                password_hash,
                updated_at_ms
            )
            VALUES (?, ?, ?)
            "#,
        )
        .bind(internal_id)
        .bind(&new_user.password_hash)
        .bind(created_at_ms)
        .execute(&mut *transaction)
        .await?;

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                u.id AS internal_id,
                u.external_id,
                u.username,
                u.normalized_username,
                pc.password_hash
            FROM users AS u
            INNER JOIN password_credentials AS pc
                ON pc.user_id = u.id
            WHERE u.id = ?
            "#,
        )
        .bind(internal_id)
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(row.into())
    }

    async fn get_user_by_username(&self, normalized_username: String) -> Result<Option<User>, AuthRepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                id AS internal_id,
                external_id,
                username,
                normalized_username
            FROM users
            WHERE normalized_username = ?
            "#,
        )
        .bind(normalized_username)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(User::from))
    }

    async fn get_user_by_internal_id(&self, internal_id: i64) -> Result<Option<User>, AuthRepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                id AS internal_id,
                external_id,
                username,
                normalized_username
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(internal_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(User::from))
    }
}

fn map_insert_user_error(error: sqlx::Error) -> AuthRepositoryError {
    match &error {
        sqlx::Error::Database(db_error)
            if db_error.is_unique_violation() => {
                AuthRepositoryError::UsernameAlreadyExists
            }
            _ => AuthRepositoryError::Database(error)
    }
}
