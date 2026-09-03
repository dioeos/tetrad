use axum::{Router, routing::post};

use crate::state::AppState;

mod register;

pub(super) fn custom_torii_auth_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register::register_handler))
}
