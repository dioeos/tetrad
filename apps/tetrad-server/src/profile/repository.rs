use async_trait::async_trait;
use thiserror::Error;

use super::model::{NewProfile, Profile};

#[derive(Debug, Error)]
pub(super) enum ProfileRepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[async_trait]
pub(super) trait ProfileRepository: Send + Sync {
    async fn create_profile(
        &self,
        new_profile: NewProfile,
    ) -> Result<Profile, ProfileRepositoryError>;
}
