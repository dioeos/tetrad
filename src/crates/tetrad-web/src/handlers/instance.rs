use axum::{Json, extract::State};
use tetrad_core::model::{ModelManager, instance::{Instance, InstanceBmc, InstanceRow}};

use crate::error::Error;

pub async fn api_get_handler(
    State(mm): State<ModelManager>
) -> Result<Json<Instance>, Error> {
    let row = InstanceBmc::get::<InstanceRow>(&mm, 1).await?;
    Ok(Json(row.try_into()?))
}
