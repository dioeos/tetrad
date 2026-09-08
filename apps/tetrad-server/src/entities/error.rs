#![allow(unused)]

use axum::{http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        debug!("{:<12} - entities::Error {self:?}", "INTO_RES");

        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
