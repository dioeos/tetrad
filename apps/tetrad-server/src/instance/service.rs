use std::{
    sync::Arc,
    error::Error
};

use thiserror::Error;

use super::{
    model::Instance,
    repository::InstanceRepository,
    use_cases::{
        EnsureInstanceExists,
        EnsureInstanceExistsError,
        GetInstance,
        GetInstanceError
    }
};

#[derive(Debug, Error)]
pub(crate) enum InstanceError {
    #[error("instance not found")]
    NotFound,

    #[error("instance storage operation failed")]
    Storage(
        #[source]
        Box<dyn Error + Send + Sync>
    ),
}

impl From<GetInstanceError> for InstanceError {
    fn from(error: GetInstanceError) -> Self {
        match error {
            GetInstanceError::NotFound => Self::NotFound,

            GetInstanceError::Repository(error) => {
                Self::Storage(Box::new(error))
            }
        }
    }
}

impl From<EnsureInstanceExistsError> for InstanceError {
    fn from(error: EnsureInstanceExistsError) -> Self {
        match error {
            EnsureInstanceExistsError::Repository(error) => {
                Self::Storage(Box::new(error))
            }
        }
    }
}

pub(crate) struct InstanceService {
    get_instance: GetInstance,
    ensure_exists_instance: EnsureInstanceExists,
}

impl InstanceService {
    pub(super) fn new(repository: Arc<dyn InstanceRepository>) -> Self {
        Self {
            get_instance: GetInstance::new(repository.clone()),
            ensure_exists_instance: EnsureInstanceExists::new(repository),
        }
    }

    pub(crate) async fn get(&self) -> Result<Instance, InstanceError> {
        Ok(self.get_instance.execute().await?)
    }

    pub(crate) async fn ensure_exists(&self, name: &str) -> Result<Instance, InstanceError> {
        Ok(self.ensure_exists_instance.execute(name).await?)
    }
}
