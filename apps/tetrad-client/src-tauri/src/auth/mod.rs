pub mod commands;

use std::sync::Arc;

use tetrad_auth::ports::{
    primary::AuthUseCases,
    secondary::IdentityProvider
};

//application layer

pub struct AuthService {
    // identity: Arc<dyn IdentityProvider>
}

#[async_trait::async_trait]
impl AuthUseCases for AuthService {
    async fn login(&self) {}
    async fn cancel_login(&self) {}
    async fn restore_session(&self) {}
    async fn logout(&self) {}
    async fn status(&self) {}
}

impl AuthService {
    pub fn new(
        // identity: Arc<dyn IdentityProvider>
    ) -> Self {
        Self { }
    }
}
