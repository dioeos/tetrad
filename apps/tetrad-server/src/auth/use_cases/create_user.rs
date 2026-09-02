use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::auth::{
    model::{CreateUserInput, NewUser, User},
    repository::{AuthRepository, AuthRepositoryError},
};

use super::util::{normalize_username, validate_username, validate_password};

//@NOTE: Currently inserting user information only occurs at the CreateUser use case,
//       which is why `InvalidUsername`, `InvalidPassword`, `UsernameAlreadyTaken`, and
//       `HashTask` errors are defined here, as they can only occur at this point
//       and are returned directly from the validators / normalizers / generators.
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
        let username = validate_username(&input.username)?;
        let password = validate_password(&input.password)?.to_owned();

        //@TODO: Integrate speed-limiters to prevent multiple concurrent password hashes
        //       spawning in multiple blocking threads (ex. 50 blocking threads will cause aggressive
        //       context switching for CPU cores)
        let password_hash = tokio::task::spawn_blocking(move || {
            password_auth::generate_hash(password)
        })
        .await?;

        let new_user = NewUser {
            external_id: Uuid::now_v7(),
            username: username.to_owned(),
            normalized_username: normalize_username(&input.username),
            password_hash
        };

        Ok(self.repository.create_user(new_user).await?)
    }
}
