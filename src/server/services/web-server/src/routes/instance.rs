use axum::{Router, routing::get};
use tetrad_core::model::ModelManager;
use tetrad_web::handlers::instance;


pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/instance", get(instance::api_get_handler))
        .with_state(mm)
}
