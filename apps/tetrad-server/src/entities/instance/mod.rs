mod dto;

use axum::{Json, extract::State};

use crate::{
    model::ModelManager
};
use super::error::Error;
use dto::InstanceDto;


pub(super) async fn get_instance_handler(
    State(mm): State<ModelManager>
) -> Result<Json<InstanceDto>, Error> {
    //use InstanceBmc
}
