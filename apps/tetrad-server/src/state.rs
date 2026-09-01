use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::{Config, instance::InstanceService};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: SqlitePool,
    pub(crate) config: Arc<Config>,
    pub(crate) instance_service: InstanceService
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
            instance_service
        }
    }
}

impl FromRef<AppState> for InstanceService {
    fn from_ref(app_state: &AppState) -> InstanceService {
        app_state.instance_service.clone()
    }
}
