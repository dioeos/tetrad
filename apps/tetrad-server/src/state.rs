use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::{Config, auth::AuthService, instance::InstanceService};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) _db: SqlitePool,
    pub(crate) _config: Arc<Config>,
    pub(crate) instance_service: InstanceService,
    pub(crate) auth_service: AuthService,
}

impl AppState {
    pub(crate) fn new(
        db: SqlitePool,
        config: Config,
        instance_service: InstanceService,
        auth_service: AuthService,
    ) -> Self {
        Self {
            _db: db,
            _config: Arc::new(config),
            instance_service,
            auth_service,
        }
    }
}

impl FromRef<AppState> for InstanceService {
    fn from_ref(app_state: &AppState) -> InstanceService {
        app_state.instance_service.clone()
    }
}

impl FromRef<AppState> for AuthService {
    fn from_ref(app_state: &AppState) -> AuthService {
        app_state.auth_service.clone()
    }
}
