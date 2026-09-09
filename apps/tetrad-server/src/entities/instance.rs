use axum::{Json, extract::State};

use super::error::Error;
use crate::model::{Instance, InstanceBmc, InstanceRow, ModelManager};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetInstancePayload {
    id: i64,
}

pub async fn api_get_instance_handler(
    State(mm): State<ModelManager>,
    Json(payload): Json<GetInstancePayload>,
) -> Result<Json<Instance>, Error> {
    let GetInstancePayload { id } = payload;

    let row: InstanceRow = InstanceBmc::get(&mm, id).await?;
    Ok(Json(row.try_into()?))
}
