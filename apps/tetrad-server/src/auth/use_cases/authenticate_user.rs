use std::sync::Arc;

use password_auth::{VerifyError, verify_password};
use thiserror::Error;

use crate::auth::{
    User,
    model::Credentials,
    repository::{AuthRepository, AuthRepositoryError},
};

use super::util::normalize_username;

const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw";

#[derive(Debug, Error)]
pub(in crate::auth) enum AuthenticateUserError {
    #[error("failed to load authentication data")]
    Repository(#[from] AuthRepositoryError),

    #[error("password verification task failed")]
    PasswordTask(#[from] tokio::task::JoinError),

    #[error("stored password hash is invalid")]
    InvalidPasswordHash(#[source] password_auth::VerifyError),
}

#[derive(Clone)]
pub(in crate::auth) struct AuthenticateUser {
    repository: Arc<dyn AuthRepository>,
}

impl AuthenticateUser {
    pub(in crate::auth) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::auth) async fn execute(
        &self,
        creds: Credentials,
    ) -> Result<Option<User>, AuthenticateUserError> {
        let normalized_username = normalize_username(&creds.username);

        let user = self
            .repository
            .get_user_by_normalized_username(&normalized_username)
            .await?;

        let password_hash = user
            .as_ref()
            .map(|user| user.password_hash.clone())
            .unwrap_or_else(|| DUMMY_PASSWORD_HASH.to_owned());

        let password = creds.password;

        let password_valid =
            tokio::task::spawn_blocking(move || match verify_password(password, &password_hash) {
                Ok(()) => Ok(true),
                Err(VerifyError::PasswordInvalid) => Ok(false),
                Err(error @ VerifyError::Parse(_)) => Err(error),
            })
            .await? //JoinError -> AuthenticateUserError::PasswordTask
            .map_err(AuthenticateUserError::InvalidPasswordHash)?;

        Ok(user.filter(|_| password_valid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::model::NewUser;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct AuthRepositoryMock {
        created_user: Option<User>,
        error: Mutex<Option<AuthRepositoryError>>,
    }

    impl AuthRepositoryMock {
        fn existing() -> Self {
            let user = User {
                internal_id: 1,
                external_id: Uuid::now_v7(),
                username: "dio_user".to_owned(),
                normalized_username: "dio_user".to_owned(),
                password_hash: password_auth::generate_hash("strongpassword"),
            };

            Self {
                created_user: Some(user),
                error: Mutex::new(None),
            }
        }

        fn returning(user: User) -> Self {
            Self {
                created_user: Some(user),
                error: Mutex::new(None),
            }
        }

        fn failing(error: AuthRepositoryError) -> Self {
            Self {
                created_user: None,
                error: Mutex::new(Some(error)),
            }
        }
    }

    #[async_trait]
    impl AuthRepository for AuthRepositoryMock {
        async fn create_user(&self, _new_user: NewUser) -> Result<User, AuthRepositoryError> {
            panic!("create_user should not be called in authenticate_user use case");
        }

        async fn get_user_by_internal_id(
            &self,
            _internal_id: i64,
        ) -> Result<Option<User>, AuthRepositoryError> {
            panic!("get_user_by_internal_id should not be called in authenticate_user use case");
        }

        async fn get_user_by_normalized_username(
            &self,
            normalized_username: &str,
        ) -> Result<Option<User>, AuthRepositoryError> {
            if let Some(error) = self.error.lock().unwrap().take() {
                return Err(error);
            }

            Ok(self
                .created_user
                .as_ref()
                .filter(|user| user.normalized_username == normalized_username)
                .cloned())
        }
    }

    #[tokio::test]
    async fn authenticates_user_with_valid_credentials() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let authenticate_user = AuthenticateUser::new(repo);

        let credentials = Credentials {
            username: " DIO_USER  ".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = authenticate_user.execute(credentials).await.unwrap();

        let user = result.expect("authentication should succeed and produce a user");

        assert_eq!(user.internal_id, 1);
        assert_eq!(user.username, "dio_user");
        assert!(password_auth::verify_password("strongpassword", &user.password_hash).is_ok());
    }

    #[tokio::test]
    async fn returns_none_when_user_does_not_exist() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let authenticate_user = AuthenticateUser::new(repo);

        let credentials = Credentials {
            username: " NO_USER ".to_owned(),
            password: "strongpassword".to_owned(),
        };

        let result = authenticate_user.execute(credentials).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn returns_none_when_invalid_password() {
        let repo = Arc::new(AuthRepositoryMock::existing());
        let authenticate_user = AuthenticateUser::new(repo);

        let credentials = Credentials {
            username: "dio_user".to_owned(),
            password: "invalidpassword".to_owned(),
        };

        let result = authenticate_user.execute(credentials).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn converts_repository_error_to_authenticate_user_repository_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));
        let authenticate_user = AuthenticateUser::new(repo);
        let credentials = Credentials {
            username: "dio_user".to_owned(),
            password: "strongpassword".to_owned(),
        };
        let result = authenticate_user.execute(credentials).await;
        assert!(matches!(result, Err(AuthenticateUserError::Repository(_))));
    }

    #[tokio::test]
    async fn converts_join_error_to_password_task_error() {
        let join_error = tokio::spawn(async move {
            panic!("simulate task failure");
        })
        .await
        .expect_err("task should fail");

        let err = AuthenticateUserError::from(join_error);
        assert!(matches!(err, AuthenticateUserError::PasswordTask(_)));
    }

    #[tokio::test]
    async fn converts_verify_error_to_invalid_password_hash_error() {
        let invalid_pw_hash_user = User {
            internal_id: 2,
            external_id: Uuid::now_v7(),
            username: "invalid_user".to_owned(),
            normalized_username: "invalid_user".to_owned(),
            password_hash: "not-a-valid-hash".to_owned()
        };
        let repo = Arc::new(AuthRepositoryMock::returning(invalid_pw_hash_user));
        let authenticate_user = AuthenticateUser::new(repo);
        let credentials = Credentials {
            username: "invalid_user".to_owned(),
            password: "not-a-valid-hash".to_owned()
        };

        let result = authenticate_user.execute(credentials).await;
        assert!(matches!(result, Err(AuthenticateUserError::InvalidPasswordHash(_))));
    }
}
