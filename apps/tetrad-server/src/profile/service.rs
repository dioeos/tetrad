use std::{error::Error, sync::Arc};

use thiserror::Error;

use super::{model::{NewProfile, Profile}, repository::{ProfileRepository}, use_cases::{CreateProfile, CreateProfileError}};

#[derive(Debug, Error)]
pub(crate) enum ProfileError {
    #[error("internal profile operation failed")]
    Internal(#[source] Box<dyn Error + Send + Sync>),
}

impl From<CreateProfileError> for ProfileError {
    fn from(error: CreateProfileError) -> Self {
        Self::Internal(Box::new(error))
    }
}

#[derive(Clone)]
pub(crate) struct ProfileService {
    create_profile: CreateProfile
}

impl ProfileService {
    pub(super) fn new(repository: Arc<dyn ProfileRepository>) -> Self {
        Self {
            create_profile: CreateProfile::new(repository)
        }
    }

    pub(crate) async fn create_profile(&self, new_profile: NewProfile) -> Result<Profile, ProfileError> {
        Ok(self.create_profile.execute(new_profile).await?)
    }
}
