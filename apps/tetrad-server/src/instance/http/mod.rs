mod error;
mod instance;

use axum::{Router, routing::get};

use crate::{state::AppState};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/instance", get(instance::get_instance))
}
