mod auth;
mod error;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub(crate) fn public_router() -> Router<AppState> {
    Router::new()
        .route("/users", post(auth::create_user))
        .route("/login", post(auth::login))
        .route("/users/{username}", get(auth::get_user_by_username))
}

pub(crate) fn protected_router() -> Router<AppState> {
    Router::new().route("/me", get(auth::me))
}
