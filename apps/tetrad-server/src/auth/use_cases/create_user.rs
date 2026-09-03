use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::auth::{
    model::{CreateUserInput, NewUser, User},
    repository::{AuthRepository, AuthRepositoryError},
};

use super::util::{normalize_username, validate_password, validate_username};

//@NOTE: Currently inserting user information only occurs at the CreateUser use case,
//       which is why `InvalidUsername`, `InvalidPassword`, `UsernameAlreadyTaken`, and
//       `HashTask` errors are defined here, as they can only occur at this point
//       and are returned directly from the validators / normalizers / generators.
#[derive(Debug, Error)]
pub(in crate::auth) enum CreateUserError {
    #[error("failed to insert the new user")]
    Repository(#[source] AuthRepositoryError),

    #[error("username is invalid")]
    InvalidUsername,

    #[error("password is invalid")]
    InvalidPassword,

    #[error("username is already taken")]
    UsernameAlreadyTaken,

    #[error("password hashing task failed")]
    HashTask(#[from] tokio::task::JoinError),
}

impl From<AuthRepositoryError> for CreateUserError {
    fn from(error: AuthRepositoryError) -> Self {
        match error {
            AuthRepositoryError::UsernameAlreadyExists => CreateUserError::UsernameAlreadyTaken,
            error => CreateUserError::Repository(error),
        }
    }
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
        let password_hash =
            tokio::task::spawn_blocking(move || password_auth::generate_hash(password)).await?;

        let new_user = NewUser {
            external_id: Uuid::now_v7(),
            username: username.to_owned(),
            normalized_username: normalize_username(&input.username),
            password_hash,
        };

        Ok(self.repository.create_user(new_user).await?)
    }
}

//@NOTE: The `CreateUser` use case is directly responsible for validation and normalization logic,
//       as the sqlite repository implementation expects already validated input when inserting logic
//       into rows. In these tests the repository is mocked in order to check the correctness of the
//       work that occurs before inserting into the database.
#[cfg(not(test))]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use password_auth::verify_password;
    use std::sync::Mutex;

    struct AuthRepositoryMock {
        created_user: Mutex<Option<User>>,
        created_error: Mutex<Option<AuthRepositoryError>>,
    }

    impl AuthRepositoryMock {
        fn succeeding() -> Self {
            Self {
                created_user: Mutex::new(None),
                created_error: Mutex::new(None),
            }
        }

        fn failing(error: AuthRepositoryError) -> Self {
            Self {
                created_user: Mutex::new(None),
                created_error: Mutex::new(Some(error)),
            }
        }

        fn take_created_user(&self) -> User {
            self.created_user
                .lock()
                .unwrap()
                .take()
                .expect("create_user should have been called")
        }
    }

    #[async_trait]
    impl AuthRepository for AuthRepositoryMock {
        async fn create_user(&self, new_user: NewUser) -> Result<User, AuthRepositoryError> {
            if let Some(error) = self.created_error.lock().unwrap().take() {
                return Err(error);
            }

            let user = User {
                internal_id: 1,
                external_id: new_user.external_id,
                username: new_user.username,
                normalized_username: new_user.normalized_username,
                password_hash: new_user.password_hash,
            };

            let mut guard = self.created_user.lock().unwrap();
            *guard = Some(user.clone());
            Ok(user)
        }
        async fn get_user_by_internal_id(
            &self,
            _internal_id: i64,
        ) -> Result<Option<User>, AuthRepositoryError> {
            panic!("get_user_by_internal_id should not be called in create_user use case");
        }
        async fn get_user_by_normalized_username(
            &self,
            _normalized_username: &str,
        ) -> Result<Option<User>, AuthRepositoryError> {
            panic!("get_user_by_normalized_username should not be called in create_user use case");
        }
    }

    #[tokio::test]
    async fn creates_user_with_expected_repository_data() {
        let repo = Arc::new(AuthRepositoryMock::succeeding());
        let create_user = CreateUser::new(repo.clone());

        let input = CreateUserInput {
            username: "    UsernamE    ".to_owned(),
            password: "correct password".to_owned(),
        };

        let user = create_user.execute(input.clone()).await.unwrap();
        let created_user = repo.take_created_user();

        assert_eq!(created_user.username, "UsernamE");
        assert_eq!(created_user.normalized_username, "username");
        assert!(!created_user.external_id.is_nil());

        assert_ne!(created_user.password_hash, "correct password");

        assert!(verify_password(input.password, &created_user.password_hash).is_ok());

        assert_eq!(created_user.internal_id, user.internal_id);
        assert_eq!(created_user.external_id, user.external_id);
    }

    #[tokio::test]
    async fn converts_repository_error_to_create_user_repository_error() {
        let repo = Arc::new(AuthRepositoryMock::failing(AuthRepositoryError::Database(
            sqlx::Error::PoolClosed,
        )));

        let create_user = CreateUser::new(repo);
        let input = CreateUserInput {
            username: "    UsernamE    ".to_owned(),
            password: "correct password".to_owned(),
        };

        let result = create_user.execute(input).await;
        assert!(matches!(result, Err(CreateUserError::Repository(_))));
    }

    #[tokio::test]
    async fn converts_join_error_to_hash_task_error() {
        let join_error = tokio::spawn(async move {
            panic!("simulate task failure");
        })
        .await
        .expect_err("task should fail");

        let err = CreateUserError::from(join_error);
        assert!(matches!(err, CreateUserError::HashTask(_)));
    }
}
