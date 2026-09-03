mod backend;
mod http;
mod model;
mod repository;
mod service;
mod sqlite;
mod use_cases;

use std::sync::Arc;

use repository::AuthRepository;
use sqlite::SqliteAuthRepository;

use sqlx::SqlitePool;

pub(crate) use backend::TetradAuthBackend;
pub(crate) use http::{protected_router, public_router};
pub(crate) use model::User;
pub(crate) use service::{AuthError, AuthService};

pub(crate) fn create_service(db: SqlitePool) -> AuthService {
    let repository: Arc<dyn AuthRepository> = Arc::new(SqliteAuthRepository::new(db));

    AuthService::new(repository)
}
