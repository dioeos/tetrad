use axum::{Router, routing::get};
use tetrad_core::model::ModelManager;
use tetrad_web::handlers::instance_handlers::api_get_instance_handler;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/instance", get(api_get_instance_handler))
        .with_state(mm)
}
