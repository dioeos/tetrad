use std::{error::Error, sync::Arc};

use thiserror::Error;

use crate::auth::model::Credentials;

use super::{
    model::{CreateUserInput, User},
    repository::AuthRepository,
    use_cases::{
        AuthenticateUser, AuthenticateUserError, CreateUser, CreateUserError, GetUser, GetUserError,
    },
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

impl From<AuthenticateUserError> for AuthError {
    fn from(error: AuthenticateUserError) -> Self {
        //@NOTE: All AuthenticateUserErrors are interpreted as internal errors.
        //       There is no InvalidCredentials conversion because that is an expected
        //       negative result, as represented by Ok(none)
        //
        //       Ok(Some(user)) -> authenticated
        //       Ok(None)       -> username or password incorrect
        //       Err(error)     -> system failure
        //
        //       The first two scenarios are expected, while the system failure is not,
        //       which is converted to and represented by AuthError::Internal and boxed
        Self::Internal(Box::new(error))
    }
}

#[derive(Clone)]
pub(crate) struct AuthService {
    create_user: CreateUser,
    get_user: GetUser,
    authenticate_user: AuthenticateUser,
}

impl AuthService {
    pub(super) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self {
            create_user: CreateUser::new(repository.clone()),
            get_user: GetUser::new(repository.clone()),
            authenticate_user: AuthenticateUser::new(repository),
        }
    }

    pub(crate) async fn create_user(&self, input: CreateUserInput) -> Result<User, AuthError> {
        Ok(self.create_user.execute(input).await?)
    }

    pub(crate) async fn get_user_by_username(&self, username: &str) -> Result<User, AuthError> {
        Ok(self.get_user.execute_by_username(username).await?)
    }

    pub(crate) async fn get_user_by_internal_id(
        &self,
        internal_id: i64,
    ) -> Result<Option<User>, AuthError> {
        Ok(self.get_user.execute_by_internal_id(internal_id).await?)
    }

    pub(crate) async fn get_user_by_normalized_username(
        &self,
        normalized_username: &str,
    ) -> Result<Option<User>, AuthError> {
        Ok(self
            .get_user
            .execute_by_normalized_username(normalized_username)
            .await?)
    }

    pub(crate) async fn authenticate_user(
        &self,
        credentials: Credentials,
    ) -> Result<Option<User>, AuthError> {
        Ok(self.authenticate_user.execute(credentials).await?)
    }
}
