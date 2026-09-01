use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::auth::{
    model::{CreateUserInput, NewUser, User},
    repository::{AuthRepository, AuthRepositoryError},
};

use super::util::{normalize_username, validate_username};

#[derive(Debug, Error)]
pub(in crate::auth) enum CreateUserError {
    #[error("failed to insert the new user")]
    Repository(#[from] AuthRepositoryError),

    #[error("username is invalid")]
    InvalidUsername,

    #[error("password is invalid")]
    InvalidPassword,

    #[error("username is already taken")]
    UsernameAlreadyTaken,

    #[error("password hashing task failed")]
    HashTask(#[from] tokio::task::JoinError),
}

#[derive(Clone)]
pub(in crate::auth) struct CreateUser {
    repository: Arc<dyn AuthRepository>,
}

impl CreateUser {
    pub(in crate::auth) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::auth) async fn execute(
        &self,
        input: CreateUserInput,
    ) -> Result<User, CreateUserError> {
        let username = validate_username(input.username.clone())?;

        let new_user = NewUser {
            external_id: Uuid::now_v7(),
            username,
            normalized_username: normalize_username(input.username),
            password_hash: "some-hash".into(),
        };

        Ok(self.repository.create_user(new_user).await?)
    }
}
