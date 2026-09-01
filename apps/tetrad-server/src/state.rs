use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{Config, instance::InstanceService};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: SqlitePool,
    pub(crate) config: Arc<Config>,
    pub(crate) instance_service: Arc<InstanceService>
}

impl AppState {
    pub(crate) fn new(
        db: SqlitePool, 
        config: Config,
        instance_service: InstanceService
    ) -> Self {
        Self {
            db,
            config: Arc::new(config),
            instance_service: Arc::new(instance_service)
        }
    }
}
