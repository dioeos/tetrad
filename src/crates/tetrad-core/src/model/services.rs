use super::error::Error;
use crate::model::{instance::Instance, ports::InstanceRepository};

#[derive(Clone)]
pub struct InstanceService<R>
where
    R: InstanceRepository,
{
    repo: R,
}

impl<R> InstanceService<R>
where
    R: InstanceRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_instance(&self) -> Result<Instance, Error> {
        self.repo.get_instance().await
    }

    pub async fn ensure_instance_exists(&self, name: String) -> Result<i64, Error> {
        self.repo.ensure_instance_exists(name).await
    }
}
