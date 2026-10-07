use axum::{Json, extract::State};
use tetrad_core::model::{instance::Instance, services::InstanceService, vendor::ModelVendor};

use crate::error::Error;

pub async fn api_get_handler(
    State(service): State<InstanceService<ModelVendor>>,
) -> Result<Json<Instance>, Error> {
    let instance = service.get_instance().await?;
    Ok(Json(instance))
}
