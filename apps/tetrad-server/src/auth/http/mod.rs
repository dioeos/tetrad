mod error;
mod auth;

use axum::{Router, routing::{get, post}};

use crate::state::AppState;

pub(crate) fn public_router() -> Router<AppState> {
    Router::new()
        .route("/user", post(auth::create_user))
        .route("/login", post(auth::login))
}

pub(crate) fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/me", get(auth::me))
}
