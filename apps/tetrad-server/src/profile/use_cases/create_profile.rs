use std::sync::Arc;

use thiserror::Error;

use crate::profile::{
    model::{NewProfile, Profile},
    repository::{ProfileRepository, ProfileRepositoryError},
};
#[derive(Debug, Error)]
pub(in crate::profile) enum CreateProfileError {
    #[error("failed to insert profile")]
    Repository(#[source] ProfileRepositoryError),
}

impl From<ProfileRepositoryError> for CreateProfileError {
    fn from(error: ProfileRepositoryError) -> Self {
        CreateProfileError::Repository(error)
    }
}

#[derive(Clone)]
pub(in crate::profile) struct CreateProfile {
    repository: Arc<dyn ProfileRepository>,
}

impl CreateProfile {
    pub(in crate::profile) fn new(repository: Arc<dyn ProfileRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::profile) async fn execute(
        &self,
        new_profile: NewProfile,
    ) -> Result<Profile, CreateProfileError> {
        Ok(self.repository.create_profile(new_profile).await?)
    }
}
