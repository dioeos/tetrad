use std::sync::Arc;

use password_auth::{VerifyError, verify_password};
use thiserror::Error;

use crate::auth::{
    User, model::Credentials, repository::{AuthRepository, AuthRepositoryError}
};

use super::util::{normalize_username};

const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw";

#[derive(Debug, Error)]
pub(in crate::auth) enum AuthenticateUserError {
    #[error("failed to load authentication data")]
    Repository(#[from] AuthRepositoryError),

    #[error("password verification task failed")]
    PasswordTask(#[from] tokio::task::JoinError),

    #[error("stored password hash is invalid")]
    InvalidPasswordHash(#[source] password_auth::VerifyError)
}

#[derive(Clone)]
pub(in crate::auth) struct AuthenticateUser {
    repository: Arc<dyn AuthRepository>
}

impl AuthenticateUser {
    pub(in crate::auth) fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::auth) async fn execute(
        &self,
        creds: Credentials
    ) -> Result<Option<User>, AuthenticateUserError>  {
        let normalized_username = normalize_username(&creds.username);

        let user = self.repository
            .get_user_by_normalized_username(&normalized_username)
            .await?;

        let password_hash = user
            .as_ref()
            .map(|user| user.password_hash.clone())
            .unwrap_or_else(|| DUMMY_PASSWORD_HASH.to_owned());

        let password = creds.password;

        let password_valid = tokio::task::spawn_blocking(move || {
            match verify_password(password, &password_hash) {
                Ok(()) => Ok(true),
                Err(VerifyError::PasswordInvalid) => Ok(false),
                Err(error @ VerifyError::Parse(_)) => Err(error)
            }
        })
        .await? //JoinError -> AuthenticateUserError::PasswordTask
        .map_err(AuthenticateUserError::InvalidPasswordHash)?;

        Ok(user.filter(|_| password_valid))
    }
}
