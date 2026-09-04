use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;
use torii::Torii;
use torii_axum::HasTorii;
use torii_storage_sqlite::SqliteRepositoryProvider;

use crate::{Config, instance::InstanceService};

type AppTorii = torii::Torii<SqliteRepositoryProvider>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) _db: SqlitePool,
    pub(crate) _config: Arc<Config>,
    pub(crate) torii: Arc<AppTorii>,
    pub(crate) instance_service: InstanceService,
}

impl AppState {
    pub(crate) fn new(
        db: SqlitePool,
        config: Config,
        torii: Arc<AppTorii>,
        instance_service: InstanceService,
    ) -> Self {
        Self {
            _db: db,
            _config: Arc::new(config),
            torii,
            instance_service,
        }
    }
}

impl HasTorii<SqliteRepositoryProvider> for AppState {
    fn torii(&self) -> &Torii<SqliteRepositoryProvider> {
        self.torii.as_ref()
    }
}

impl FromRef<AppState> for InstanceService {
    fn from_ref(app_state: &AppState) -> InstanceService {
        app_state.instance_service.clone()
    }
}
