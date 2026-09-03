use std::sync::Arc;

use thiserror::Error;

use crate::auth::{
    model::User,
    repository::{AuthRepository, AuthRepositoryError},
    use_cases::util::normalize_username,
};

#[derive(Debug, Error)]
pub(in crate::auth) enum GetUserError {
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

    pub(in crate::auth) async fn execute_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, GetUserError> {
        let normalized_username = normalize_username(username);
        Ok(self
            .repository
            .get_user_by_normalized_username(&normalized_username)
            .await?)
        //@NOTE: Does not transform Option<T> into Result<T, E>  with .ok_or(GetUserError::NotFound) in order to satisfy
        //       the expection of Result<Option<User>> in axum_login's AuthnBackend trait (`authenticate` fn).
        //       This is the same for the rest of the `get_user_by_internal_id` chain. The ? operator
        //       ensures that the `AuthRepositoryError` is converted to a `GetUserError::Repository`
    }
}

#[cfg(not(test))]
mod tests {
    use super::*;
    use crate::auth::model::NewUser;
    use async_trait::async_trait;
    use std::sync::Mutex;
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
                username: "Username".to_owned(),
                normalized_username: "username".to_owned(),
                password_hash: password_auth::generate_hash("password"),
            };

            Self {
                existing_user: Some(user),
                error: Mutex::new(None),
            }
        }

        fn empty() -> Self {
            Self {
                existing_user: None,
                error: Mutex::new(None),
            }
        }

        fn failing(error: AuthRepositoryError) -> Self {
            Self {
                existing_user: None,
                error: Mutex::new(Some(error)),
            }
        }
    }

    #[async_trait]
    impl AuthRepository for AuthRepositoryMock {
        async fn create_user(&self, _new_user: NewUser) -> Result<User, AuthRepositoryError> {
            panic!("create_user should not be called in get_user use case");
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
    async fn by_normalized_username_returns_user() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let get_user = GetUser::new(repo);

        let user = get_user
            .execute_by_username("username")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.internal_id, 1);
        assert_eq!(user.username, "Username");
        assert!(password_auth::verify_password("password", &user.password_hash).is_ok());
    }

    #[tokio::test]
    async fn by_internal_id_returns_user() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let get_user = GetUser::new(repo);

        let user = get_user.execute_by_internal_id(1).await.unwrap().unwrap();
        assert_eq!(user.internal_id, 1);
        assert_eq!(user.username, "Username");
        assert!(password_auth::verify_password("password", &user.password_hash).is_ok());
    }

    #[tokio::test]
    async fn by_normalized_username_returns_none_when_user_does_not_exist() {
        let repo = Arc::new(AuthRepositoryMock::empty());
        let get_user = GetUser::new(repo);

        let user = get_user.execute_by_username("username").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn by_internal_id_returns_none_when_user_does_not_exist() {
        let repo = Arc::new(AuthRepositoryMock::empty());
        let get_user = GetUser::new(repo);

        let user = get_user.execute_by_internal_id(1).await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn by_normalized_username_converts_auth_repository_error_to_get_user_repository_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let get_user = GetUser::new(repo);

        let result = get_user.execute_by_username("username").await;

        assert!(matches!(result, Err(GetUserError::Repository(_))));
    }

    #[tokio::test]
    async fn by_internal_id_converts_auth_repository_error_to_get_user_repository_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let get_user = GetUser::new(repo);

        let result = get_user.execute_by_internal_id(1).await;

        assert!(matches!(result, Err(GetUserError::Repository(_))));
    }
}
