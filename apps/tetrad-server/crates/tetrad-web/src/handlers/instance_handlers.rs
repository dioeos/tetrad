use axum::{Json, extract::State};

use crate::error::Error;
use tetrad_core::model::{
    ModelManager,
    instance::{Instance, InstanceBmc, InstanceRow},
};

pub async fn api_get_instance_handler(
    State(mm): State<ModelManager>,
) -> Result<Json<Instance>, Error> {
    let row: InstanceRow = InstanceBmc::get(&mm, 1).await?;
    Ok(Json(row.try_into()?))
}
