use axum_login::{AuthnBackend, UserId};

use super::{
    AuthError, AuthService,
    model::{Credentials, User},
};

#[derive(Clone)]
pub(crate) struct TetradAuthBackend {
    auth_service: AuthService,
}

impl TetradAuthBackend {
    pub(crate) fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }
}

impl AuthnBackend for TetradAuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        self.auth_service.authenticate_user(creds).await
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.auth_service.get_user_by_internal_id(*user_id).await
    }
}

// @NOTE: The conversions of the errors that come from the auth service's use cases
//        that are used here (`authenticate_user` & `get_user_by_internal_id`) are
//        tested at the service level boundary
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{
        model::NewUser,
        repository::{AuthRepository, AuthRepositoryError},
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct AuthRepositoryMock {
        existing_user: Option<User>,
        error: Mutex<Option<AuthRepositoryError>>,
    }

    impl AuthRepositoryMock {
        fn existing() -> Self {
            let user = User {
                internal_id: 1,
                external_id: Uuid::now_v7(),
                username: "username".to_owned(),
                normalized_username: "username".to_owned(),
                password_hash: password_auth::generate_hash("password"),
            };

            Self {
                existing_user: Some(user),
                error: Mutex::new(None),
            }
        }

        fn failing(error: AuthRepositoryError) -> Self {
            Self {
                existing_user: None,
                error: Mutex::new(Some(error))
            }
        }
    }
    #[async_trait]
    impl AuthRepository for AuthRepositoryMock {
        async fn create_user(&self, _new_user: NewUser) -> Result<User, AuthRepositoryError> {
            panic!("create_user should not be called in auth backend");
        }

        async fn get_user_by_internal_id(
            &self,
            internal_id: i64,
        ) -> Result<Option<User>, AuthRepositoryError> {
            if let Some(error) = self.error.lock().unwrap().take() {
                return Err(error);
            }

            Ok(self
                .existing_user
                .as_ref()
                .filter(|user| user.internal_id == internal_id)
                .cloned())
        }

        async fn get_user_by_normalized_username(
            &self,
            normalized_username: &str,
        ) -> Result<Option<User>, AuthRepositoryError> {
            if let Some(error) = self.error.lock().unwrap().take() {
                return Err(error);
            }

            Ok(self
                .existing_user
                .as_ref()
                .filter(|user| user.normalized_username == normalized_username)
                .cloned())
        }
    }

    #[tokio::test]
    async fn authenticate_returns_user_on_valid_credentials() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let creds = Credentials {
            username: "username".to_owned(),
            password: "password".to_owned()
        };

        let user = backend.authenticate(creds).await.unwrap().unwrap();

        assert_eq!(user.username, "username");
        assert_eq!(user.internal_id, 1);
    }

    #[tokio::test]
    async fn authenticate_returns_none_on_invalid_credentials() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let creds = Credentials {
            username: "username".to_owned(),
            password: "wrongpassword".to_owned()
        };

        let result = backend.authenticate(creds).await.unwrap();
        assert!(result.is_none())
    }

    #[tokio::test]
    async fn authenticate_converts_authenticate_user_repository_error_to_internal_auth_error() {
        let repo = Arc::new(
            AuthRepositoryMock::failing(
                AuthRepositoryError::Database(sqlx::Error::PoolClosed)
            )
        );
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let creds = Credentials {
            username: "username".to_owned(),
            password: "password".to_owned()
        };

        let result = backend.authenticate(creds).await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }

    #[tokio::test]
    async fn get_user_returns_user_on_existing_user() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let user = backend.get_user(&1).await.unwrap().unwrap();

        assert_eq!(user.username, "username");
        assert_eq!(user.internal_id, 1);
    }

    #[tokio::test]
    async fn get_user_returns_none_on_no_existing_user() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let result = backend.get_user(&2).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_user_converts_get_user_repository_error_to_internal_auth_error() {
        let repo = Arc::new(
            AuthRepositoryMock::failing(
                AuthRepositoryError::Database(sqlx::Error::PoolClosed)
            )
        );
        let auth_service = AuthService::new(repo);
        let backend = TetradAuthBackend::new(auth_service);

        let result = backend.get_user(&1).await;

        assert!(matches!(result, Err(AuthError::Internal(_))));
    }
}
