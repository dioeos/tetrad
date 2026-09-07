mod instance;
mod error;

use axum::Router;

use crate::{
    model::ModelManager
};

pub fn instance_routes(mm: ModelManager) -> Router {
    Router::new()
        .with_state(mm)
}
