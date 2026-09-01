use std::sync::Arc;

use thiserror::Error;

use crate::instance::{
    model::Instance,
    repository::{InstanceRepository, RepositoryError},
};

#[derive(Debug, Error)]
pub(in crate::instance) enum GetInstanceError {
    #[error("instance not found")]
    NotFound,

    #[error("failed to load instance")]
    Repository(#[from] RepositoryError),
}

#[derive(Clone)]
pub(in crate::instance) struct GetInstance {
    repository: Arc<dyn InstanceRepository>,
}

impl GetInstance {
    pub(in crate::instance) fn new(repository: Arc<dyn InstanceRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::instance) async fn execute(&self) -> Result<Instance, GetInstanceError> {
        self.repository
            .get()
            .await?
            .ok_or(GetInstanceError::NotFound)
    }
}
