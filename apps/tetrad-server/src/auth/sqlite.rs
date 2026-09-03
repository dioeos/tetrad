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
    external_id: String,
    username: String,
    normalized_username: String,
    password_hash: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            internal_id: row.internal_id,
            external_id: Uuid::parse_str(&row.external_id)
                .expect("stored user external_id must be a UUID"),
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
        .bind(new_user.external_id.to_string())
        .bind(new_user.username)
        .bind(new_user.normalized_username)
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

    async fn get_user_by_internal_id(
        &self,
        internal_id: i64,
    ) -> Result<Option<User>, AuthRepositoryError> {
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
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(User::from))
    }

    async fn get_user_by_normalized_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<User>, AuthRepositoryError> {
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
            WHERE u.normalized_username = ?
            "#,
        )
        .bind(normalized_username)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(User::from))
    }
}

fn map_insert_user_error(error: sqlx::Error) -> AuthRepositoryError {
    match &error {
        sqlx::Error::Database(db_error) if db_error.is_unique_violation() => {
            AuthRepositoryError::UsernameAlreadyExists
        }
        _ => AuthRepositoryError::Database(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_user_row_len(pool: SqlitePool) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap()
    }

    async fn insert_existing_user(pool: SqlitePool) {
        let now = time::now_ms();
        sqlx::query(
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
        .bind(uuid::Uuid::now_v7().to_string())
        .bind("username".to_owned())
        .bind("username".to_owned())
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    }

    //create_user tests
    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_user_on_valid_data(pool: SqlitePool) {
        let repo = SqliteAuthRepository::new(pool);
        let new = NewUser {
            username: "username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password"),
        };
        let user = repo.create_user(new).await.unwrap();
        assert_eq!(user.username, "username");
        assert!(!user.external_id.is_nil());
        assert_eq!(user.normalized_username, "username");
        assert!(!user.password_hash.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_username_already_exists_error_when_normalize_username_is_duplicate(
        pool: SqlitePool,
    ) {
        let created_at_ms = time::now_ms();
        sqlx::query(
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
        .bind(uuid::Uuid::now_v7().to_string())
        .bind("Username")
        .bind("username")
        .bind(created_at_ms)
        .bind(created_at_ms)
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqliteAuthRepository::new(pool.clone());
        let new = NewUser {
            username: "Username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password"),
        };

        let result = repo.create_user(new).await;
        assert!(matches!(
            result,
            Err(AuthRepositoryError::UsernameAlreadyExists)
        ));

        let row_length = get_user_row_len(pool).await;

        assert_eq!(row_length, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_rolls_back_user_insert_when_password_credentials_insert_fails(
        pool: SqlitePool,
    ) {
        sqlx::query(
            r#"
            CREATE TRIGGER fail_password_credentials_insert
            BEFORE INSERT ON password_credentials
            BEGIN
                SELECT RAISE(ABORT, 'simulated password credentials insert failure');
            END
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqliteAuthRepository::new(pool.clone());
        let new = NewUser {
            username: "Username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password"),
        };
        let result = repo.create_user(new).await;
        assert!(matches!(result, Err(AuthRepositoryError::Database(_))));
        let row_length = get_user_row_len(pool).await;
        assert_eq!(row_length, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_repository_error_when_db_begin_transaction_fails(
        pool: SqlitePool,
    ) {
        let repo = SqliteAuthRepository::new(pool.clone());
        pool.close().await;

        let new = NewUser {
            username: "Username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password"),
        };
        let result = repo.create_user(new).await;
        assert!(matches!(
            result,
            Err(AuthRepositoryError::Database(sqlx::Error::PoolClosed))
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_repository_error_when_fetch_created_user_fails(pool: SqlitePool) {
        sqlx::query(
            r#"
            CREATE TRIGGER simulate_final_user_fetch_failure
            AFTER INSERT ON password_credentials
            BEGIN
                DELETE FROM password_credentials
                WHERE user_id = NEW.user_id;
            END
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqliteAuthRepository::new(pool.clone());
        let new = NewUser {
            username: "Username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password"),
        };
        let result = repo.create_user(new).await;
        assert!(matches!(
            result,
            Err(AuthRepositoryError::Database(sqlx::Error::RowNotFound))
        ));
        let row_length = get_user_row_len(pool).await;
        assert_eq!(row_length, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_repository_error_when_commit_fails(pool: SqlitePool) {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("CREATE TABLE parent (id INTEGER PRIMARY KEY);")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE child (
                id INTEGER PRIMARY KEY,
                parent_id INTEGER,
                FOREIGN KEY (parent_id) REFERENCES parent(id) DEFERRABLE INITIALLY DEFERRED
            );
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TRIGGER simulate_commit_failure
            AFTER INSERT ON password_credentials
            BEGIN
                INSERT INTO child (parent_id) VALUES (99999);
            END;
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = SqliteAuthRepository::new(pool.clone());
        let new = NewUser {
            username: "Username".to_owned(),
            external_id: uuid::Uuid::now_v7(),
            normalized_username: "username".to_owned(),
            password_hash: password_auth::generate_hash("password")
        };
        let result = repo.create_user(new).await;

        assert!(matches!(
                result,
                Err(AuthRepositoryError::Database(_))
        ));
        let row_length = get_user_row_len(pool).await;
        assert_eq!(row_length, 0);
    }

    //get_user_by_internal_id tests
    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_by_internal_id_returns_user_when_id_exists(pool: SqlitePool) {
        insert_existing_user(pool.clone()).await;
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_by_internal_id_returns_user_when_id_does_not_exist(pool: SqlitePool) {
        insert_existing_user(pool.clone()).await;
    }

    //get_user_by_normalized_username tests
    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_by_normalized_username_returns_user_when_username_exists(pool: SqlitePool) {
        insert_existing_user(pool.clone()).await;
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_by_normalized_username_returns_user_when_username_does_not_exist(
        pool: SqlitePool,
    ) {
        insert_existing_user(pool.clone()).await;
    }
}
