use async_trait::async_trait;

use super::{error::Error, instance::Instance};

#[async_trait]
pub trait InstanceRepository: Clone + Send + Sync + 'static {
    async fn get_instance(&self) -> Result<Instance, Error>;
    async fn ensure_instance_exists(&self, name: String) -> Result<i64, Error>;
}
