use async_trait::async_trait;

use super::error::Error;
use crate::model::{
    ModelManager,
    instance::{Instance, InstanceBmc, InstanceForCreate, InstanceRow},
    ports::InstanceRepository,
};

#[derive(Clone)]
pub struct ModelVendor {
    manager: ModelManager,
}

impl ModelVendor {
    pub fn new(manager: ModelManager) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl InstanceRepository for ModelVendor {
    async fn get_instance(&self) -> Result<Instance, Error> {
        let row = InstanceBmc::get::<InstanceRow>(&self.manager, 1).await?;
        row.try_into()
    }

    async fn ensure_instance_exists(&self, name: String) -> Result<i64, Error> {
        InstanceBmc::ensure_exists(&self.manager, InstanceForCreate { name }).await
    }
}
