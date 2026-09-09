use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse};
use tetrad_core::model;
use tracing::debug;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    Model(#[from] model::Error),
}

//@NOTE: Converts tetrad_web::errors into internal HTTP responses.
//       These internal HTTP responses are then transformed in the
//       middleware layer, where the response is mapped to a specific
//       client-side error
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        debug!("{:<12} - entities::Error {self:?}", "INTO_RES");

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
