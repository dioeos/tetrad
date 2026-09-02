use std::sync::Arc;

use thiserror::Error;

use crate::auth::{
    model::User,
    repository::{AuthRepository, AuthRepositoryError},
};

use super::util::normalize_username;

#[derive(Debug, Error)]
pub(in crate::auth) enum GetUserError {
    #[error("user not found")]
    NotFound,

    #[error("failed to fetch user")]
    Repository(#[from] AuthRepositoryError),
}

#[derive(Clone)]
pub(in crate::auth) struct GetUser {
    repository: Arc<dyn AuthRepository>,
}

impl GetUser {
    pub(in crate::auth) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::auth) async fn execute_by_username(
        &self,
        username: &str,
    ) -> Result<User, GetUserError> {
        let normalized_username: String = normalize_username(username);

        self.repository
            .get_user_by_username(normalized_username)
            .await?
            .ok_or(GetUserError::NotFound)
    }

    pub(in crate::auth) async fn execute_by_internal_id(
        &self,
        internal_id: i64,
    ) -> Result<Option<User>, GetUserError> {
        Ok(self.repository.get_user_by_internal_id(internal_id).await?)
        //@NOTE: Does not transform Option<T> into Result<T, E>  with .ok_or(GetUserError::NotFound) in order to satisfy
        //       the expection of Result<Option<User>> in axum_login's AuthnBackend trait (`get_user` fn). This is
        //       the same for the rest of the `get_user_by_internal_id` chain. The ? operator
        //       ensures that the `AuthRepositoryError` is converted to a `GetUserError::Repository`
    }

    pub(in crate::auth) async fn execute_by_normalized_username(
        &self,
        normalized_username: &str
    ) -> Result<Option<User>, GetUserError> {
        Ok(self.repository.get_user_by_normalized_username(normalized_username).await?)
        //@NOTE: Does not transform Option<T> into Result<T, E>  with .ok_or(GetUserError::NotFound) in order to satisfy
        //       the expection of Result<Option<User>> in axum_login's AuthnBackend trait (`authenticate` fn). 
        //       This is the same for the rest of the `get_user_by_internal_id` chain. The ? operator
        //       ensures that the `AuthRepositoryError` is converted to a `GetUserError::Repository`
    }
}
