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

    pub(crate) async fn get_user_by_internal_id(
        &self,
        internal_id: i64,
    ) -> Result<Option<User>, AuthError> {
        Ok(self.get_user.execute_by_internal_id(internal_id).await?)
    }

    pub(crate) async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, AuthError> {
        Ok(self
            .get_user
            .execute_by_username(username)
            .await?)
    }

    pub(crate) async fn authenticate_user(
        &self,
        credentials: Credentials,
    ) -> Result<Option<User>, AuthError> {
        Ok(self.authenticate_user.execute(credentials).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    use crate::auth::{model::NewUser, repository::AuthRepositoryError};

    struct AuthRepositoryMock {
        error: Mutex<Option<AuthRepositoryError>>,
    }

    impl AuthRepositoryMock {
        fn failing(error: AuthRepositoryError) -> Self {
            Self {
                error: Mutex::new(Some(error)),
            }
        }

        fn empty() -> Self {
            Self {
                error: Mutex::new(None),
            }
        }

        fn take_error(&self, method: &str) -> AuthRepositoryError {
            self.error
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| panic!("unexpected repository call: {method}"))
        }
    }

    #[async_trait]
    impl AuthRepository for AuthRepositoryMock {
        async fn create_user(&self, _new_user: NewUser) -> Result<User, AuthRepositoryError> {
            Err(self.take_error("create_user"))
        }

        async fn get_user_by_internal_id(
            &self,
            _internal_id: i64,
        ) -> Result<Option<User>, AuthRepositoryError> {
            Err(self.take_error("get_user_by_internal_id"))
        }

        async fn get_user_by_normalized_username(
            &self,
            _normalized_username: &str,
        ) -> Result<Option<User>, AuthRepositoryError> {
            Err(self.take_error("get_user_by_normalized_username"))
        }
    }

    #[tokio::test]
    async fn converts_get_user_by_internal_id_repository_error_to_internal_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let service = AuthService::new(repo);

        let result = service.get_user_by_internal_id(1).await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }

    #[tokio::test]
    async fn converts_get_user_by_username_repository_error_to_internal_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let service = AuthService::new(repo);

        let result = service.get_user_by_username("username").await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }

    #[tokio::test]
    async fn converts_create_user_repository_error_to_internal_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let service = AuthService::new(repo);

        let input = CreateUserInput {
            username: "username".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = service.create_user(input).await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }

    #[tokio::test]
    async fn converts_invalid_username_to_invalid_username_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::empty());
        let service = AuthService::new(repo);

        let input = CreateUserInput {
            username: "invalid username!".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = service.create_user(input).await;

        assert!(matches!(result, Err(AuthError::InvalidUsername)));
    }

    #[tokio::test]
    async fn converts_invalid_password_to_invalid_password_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::empty());
        let service = AuthService::new(repo);

        let input = CreateUserInput {
            username: "username".to_owned(),
            password: "a".to_owned(),
        };

        let result = service.create_user(input).await;

        assert!(matches!(result, Err(AuthError::InvalidPassword)));
    }

    #[tokio::test]
    async fn converts_username_already_exists_to_username_already_taken_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(
            AuthRepositoryError::UsernameAlreadyExists,
        ));
        let service = AuthService::new(repo);

        let input = CreateUserInput {
            username: "username".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = service.create_user(input).await;

        assert!(matches!(result, Err(AuthError::UsernameAlreadyTaken)));
    }

    #[tokio::test]
    async fn converts_authenticate_user_repository_error_to_internal_auth_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let service = AuthService::new(repo);

        let credentials = Credentials {
            username: "username".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = service.authenticate_user(credentials).await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }

    #[tokio::test]
    async fn converts_create_user_hash_task_error_to_internal_auth_error() {
        let join_error = tokio::spawn(async {
            panic!("simulated password hashing task failure");
        })
        .await
        .unwrap_err();

        let error = CreateUserError::HashTask(join_error);
        let result = AuthError::from(error);

        assert!(matches!(result, AuthError::Internal(_)));
    }
}
