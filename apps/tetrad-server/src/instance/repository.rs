use async_trait::async_trait;
use thiserror::Error;

use super::model::Instance;

#[derive(Debug, Error)]
pub(super) enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error)
}

#[async_trait]
pub(super) trait InstanceRepository: Send + Sync {
    async fn get(
        &self
    ) -> Result<Option<Instance>, RepositoryError>;

    async fn ensure_exists(
        &self,
        name: &str
    ) -> Result<Instance, RepositoryError>;
}
