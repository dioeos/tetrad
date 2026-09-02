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
