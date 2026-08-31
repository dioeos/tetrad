use std::sync::Arc;

use thiserror::Error;

use crate::instance::{
    model::Instance,
    repository::{InstanceRepository, RepositoryError},
};

#[derive(Debug, Error)]
pub(in crate::instance) enum EnsureInstanceExistsError {
    #[error("failed to ensure instance exists")]
    Repository(#[from] RepositoryError),
}

pub(in crate::instance) struct EnsureInstanceExists {
    repository: Arc<dyn InstanceRepository>,
}

impl EnsureInstanceExists {
    pub(in crate::instance) fn new(repository: Arc<dyn InstanceRepository>) -> Self {
        Self { repository }
    }

    pub(in crate::instance) async fn execute(
        &self,
        name: &str,
    ) -> Result<Instance, EnsureInstanceExistsError> {
        Ok(self.repository.ensure_exists(name).await?)
    }
}
