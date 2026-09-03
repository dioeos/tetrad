use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;
use torii::Torii;
use torii_axum::HasTorii;
use torii_storage_seaorm::SeaORMRepositoryProvider;

use crate::{Config, instance::InstanceService, profile::ProfileService};

type AppTorii = torii::Torii<torii_storage_seaorm::SeaORMRepositoryProvider>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) _db: SqlitePool,
    pub(crate) _config: Arc<Config>,
    pub(crate) torii: Arc<AppTorii>,
    pub(crate) instance_service: InstanceService,
    pub(crate) profile_service: ProfileService
}

impl AppState {
    pub(crate) fn new(
        db: SqlitePool,
        config: Config,
        torii: Arc<AppTorii>,
        instance_service: InstanceService,
        profile_service: ProfileService
    ) -> Self {
        Self {
            _db: db,
            _config: Arc::new(config),
            torii,
            instance_service,
            profile_service
        }
    }
}

impl HasTorii<SeaORMRepositoryProvider> for AppState {
    fn torii(&self) -> &Torii<SeaORMRepositoryProvider> {
        self.torii.as_ref()
    }
}

impl FromRef<AppState> for InstanceService {
    fn from_ref(app_state: &AppState) -> InstanceService {
        app_state.instance_service.clone()
    }
}

impl FromRef<AppState> for ProfileService {
    fn from_ref(app_state: &AppState) -> ProfileService {
        app_state.profile_service.clone()
    }
}
