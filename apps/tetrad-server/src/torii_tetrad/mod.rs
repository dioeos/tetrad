use axum::{Router, routing::post};
use tetrad_api_contract::{Endpoint, RegisterEndpoint};

use crate::state::AppState;

mod register;
mod login;

pub(super) fn custom_torii_auth_router() -> Router<AppState> {
    Router::new().route(RegisterEndpoint::CONTRACT.path, post(register::register_handler))
}
