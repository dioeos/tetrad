use super::error::InstanceHttpError;
use crate::{instance::Instance, state::AppState};

use axum::{Json, extract::State};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct InstanceDto {
    id: String,
    name: String,
    setup_completed_at_ms: Option<i64>,
}

impl From<Instance> for InstanceDto {
    fn from(instance: Instance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            setup_completed_at_ms: instance.setup_completed_at_ms,
        }
    }
}

pub(crate) async fn get_instance(
    State(state): State<AppState>,
) -> Result<Json<InstanceDto>, InstanceHttpError> {
    let instance = state.instance_service.get().await?;
    Ok(Json(instance.into()))
}
