mod model;
mod repository;
mod service;
mod sqlite;
mod use_cases;

use std::sync::Arc;

use repository::ProfileRepository;
use sqlite::SqliteProfileRepository;
use sqlx::SqlitePool;

pub(crate) use model::NewProfile;
pub(crate) use service::ProfileService;

pub(crate) fn create_service(db: SqlitePool) -> ProfileService {
    let repository: Arc<dyn ProfileRepository> = Arc::new(SqliteProfileRepository::new(db));

    ProfileService::new(repository)
}
