use axum_login::AuthUser;
use serde::Deserialize;
use uuid::Uuid;

pub(in crate::auth) struct NewUser {
    pub(in crate::auth) external_id: Uuid,
    pub(in crate::auth) username: String,
    pub(in crate::auth) normalized_username: String,
    pub(in crate::auth) password_hash: String,
}

#[derive(Clone)]
pub(crate) struct CreateUserInput {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Clone)]
pub(crate) struct User {
    pub(crate) internal_id: i64,
    pub(crate) external_id: Uuid,
    pub(crate) username: String,
    pub(crate) normalized_username: String,
    pub(crate) password_hash: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("internal_id", &self.internal_id)
            .field("external_id", &self.external_id)
            .field("username", &self.username)
            .field("normalized_username", &self.normalized_username)
            .field("password_hash", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    //Backend operates on the internal_id
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.internal_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Clone, Deserialize)]
pub(crate) struct Credentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}
