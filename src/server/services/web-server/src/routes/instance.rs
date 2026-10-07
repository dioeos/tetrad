use axum::{Router, routing::get};
use tetrad_core::model::{services::InstanceService, vendor::ModelVendor};
use tetrad_web::handlers::instance;


pub fn routes(instance_service: InstanceService<ModelVendor>) -> Router {
    Router::new()
        .route("/instance", get(instance::api_get_handler))
        .with_state(instance_service)
}
