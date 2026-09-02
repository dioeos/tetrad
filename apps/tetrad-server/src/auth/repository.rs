use async_trait::async_trait;
use thiserror::Error;

use super::model::{NewUser, User};

#[derive(Debug, Error)]
pub(super) enum AuthRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("username already exists")]
    UsernameAlreadyExists,
}

#[async_trait]
pub(super) trait AuthRepository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<User, AuthRepositoryError>;

    async fn get_user_by_username(
        &self,
        normalized_username: String,
    ) -> Result<Option<User>, AuthRepositoryError>;

    async fn get_user_by_internal_id(
        &self,
        internal_id: i64,
    ) -> Result<Option<User>, AuthRepositoryError>;

    async fn get_user_by_normalized_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<User>, AuthRepositoryError>;
}
