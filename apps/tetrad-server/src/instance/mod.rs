//@NOTE: `instance` INTERNALS
mod model;
mod repository;
mod service;
mod sqlite;
mod use_cases;
mod http;

use std::sync::Arc;

use repository::InstanceRepository;
use sqlite::SqliteInstanceRepository;

use sqlx::SqlitePool;

//@NOTE: PUBLIC API OF `instance`
pub(crate) use service::{InstanceService, InstanceError};
pub(crate) use model::Instance;
pub(crate) use http::router;

pub(crate) fn create_service(db: SqlitePool) -> InstanceService {
    let repository: Arc<dyn InstanceRepository> = Arc::new(SqliteInstanceRepository::new(db));

    InstanceService::new(repository)
}
