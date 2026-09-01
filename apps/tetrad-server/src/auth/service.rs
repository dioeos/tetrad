use std::{error::Error, sync::Arc};

use thiserror::Error;

use super::{
    model::{CreateUserInput, User},
    repository::AuthRepository,
    use_cases::{CreateUser, CreateUserError, GetUser, GetUserError},
};

#[derive(Debug, Error)]
pub(crate) enum AuthError {
    #[error("internal authentication operation failed")]
    Internal(#[source] Box<dyn Error + Send + Sync>),

    #[error("user not found")]
    UserNotFound,

    #[error("invalid username")]
    InvalidUsername,

    #[error("invalid password")]
    InvalidPassword,

    #[error("username is already taken")]
    UsernameAlreadyTaken,
}

impl From<CreateUserError> for AuthError {
    fn from(error: CreateUserError) -> Self {
        match error {
            CreateUserError::InvalidUsername => Self::InvalidUsername,
            CreateUserError::InvalidPassword => Self::InvalidPassword,
            CreateUserError::UsernameAlreadyTaken => Self::UsernameAlreadyTaken,
            error @ (CreateUserError::Repository(_) | CreateUserError::HashTask(_)) => {
                Self::Internal(Box::new(error))
            }
        }
    }
}

impl From<GetUserError> for AuthError {
    fn from(error: GetUserError) -> Self {
        match error {
            GetUserError::NotFound => Self::UserNotFound,
            error @ GetUserError::Repository(_) => Self::Internal(Box::new(error)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct AuthService {
    create_user: CreateUser,
    get_user: GetUser,
}

impl AuthService {
    pub(super) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self {
            create_user: CreateUser::new(repository.clone()),
            get_user: GetUser::new(repository),
        }
    }

    pub(crate) async fn create_user(&self, input: CreateUserInput) -> Result<User, AuthError> {
        Ok(self.create_user.execute(input).await?)
    }

    pub(crate) async fn get_user_by_username(&self, username: String) -> Result<User, AuthError> {
        Ok(self.get_user.execute_by_username(username).await?)
    }

    pub(crate) async fn get_user_by_internal_id(&self, internal_id: i64) -> Result<Option<User>, AuthError> {
        Ok(self.get_user.execute_by_internal_id(internal_id).await?)
    }
}
