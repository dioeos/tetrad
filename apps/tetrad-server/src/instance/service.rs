use std::{error::Error, sync::Arc};

use thiserror::Error;

use super::{
    model::Instance,
    repository::InstanceRepository,
    use_cases::{EnsureInstanceExists, EnsureInstanceExistsError, GetInstance, GetInstanceError},
};

#[derive(Debug, Error)]
pub(crate) enum InstanceError {
    #[error("instance not found")]
    NotFound,

    #[error("instance storage operation failed")]
    Storage(#[source] Box<dyn Error + Send + Sync>),
}

impl From<GetInstanceError> for InstanceError {
    fn from(error: GetInstanceError) -> Self {
        match error {
            GetInstanceError::NotFound => Self::NotFound,

            GetInstanceError::Repository(error) => Self::Storage(Box::new(error)),
        }
    }
}

impl From<EnsureInstanceExistsError> for InstanceError {
    fn from(error: EnsureInstanceExistsError) -> Self {
        match error {
            EnsureInstanceExistsError::Repository(error) => Self::Storage(Box::new(error)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InstanceService {
    //service use cases
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

#[cfg(test)]
mod tests {
    use super::{InstanceError, InstanceService};
    use crate::instance::{
        model::Instance,
        repository::{InstanceRepository, RepositoryError},
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    struct EmptyRepository;

    #[async_trait]
    impl InstanceRepository for EmptyRepository {
        async fn get(&self) -> Result<Option<Instance>, RepositoryError> {
            Ok(None)
        }

        async fn ensure_exists(&self, _name: &str) -> Result<Instance, RepositoryError> {
            unreachable!("this test repository only supports get()")
        }
    }

    struct BrokenRepository;

    #[async_trait]
    impl InstanceRepository for BrokenRepository {
        async fn get(&self) -> Result<Option<Instance>, RepositoryError> {
            Err(RepositoryError::Database(sqlx::Error::PoolClosed))
        }

        async fn ensure_exists(&self, _name: &str) -> Result<Instance, RepositoryError> {
            Err(RepositoryError::Database(sqlx::Error::PoolClosed))
        }
    }

    #[tokio::test]
    async fn get_maps_missing_instance_to_not_found() {
        let service = InstanceService::new(Arc::new(EmptyRepository));
        let result = service.get().await;

        assert!(matches!(result, Err(InstanceError::NotFound)));
    }

    #[tokio::test]
    async fn get_maps_repository_failure_to_storage_error() {
        let service = InstanceService::new(Arc::new(BrokenRepository));
        let result = service.get().await;

        assert!(matches!(result, Err(InstanceError::Storage(_))));
    }

    #[tokio::test]
    async fn ensure_exists_maps_repository_failure_to_storage_error() {
        let service = InstanceService::new(Arc::new(BrokenRepository));
        let result = service.ensure_exists("test-instance").await;

        assert!(matches!(result, Err(InstanceError::Storage(_))));
    }
}
