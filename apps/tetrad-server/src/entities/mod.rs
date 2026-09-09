#![allow(unused)]

mod error;
mod instance;

use axum::{Router, routing::get};

use crate::model::ModelManager;

pub fn instance_routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/instance", get(instance::api_get_instance_handler))
        .with_state(mm)
}
