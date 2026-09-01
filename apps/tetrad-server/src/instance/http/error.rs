use crate::instance::InstanceError;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
struct InstanceErrorDto {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub(crate) struct InstanceHttpError {
    status: StatusCode,
    body: InstanceErrorDto,
}

impl InstanceHttpError {
    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: InstanceErrorDto {
                code: "not_found",
                message,
            },
        }
    }

    fn internal_server_error(message: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: InstanceErrorDto {
                code: "internal_server_error",
                message,
            },
        }
    }
}

impl IntoResponse for InstanceHttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<InstanceError> for InstanceHttpError {
    fn from(error: InstanceError) -> Self {
        match error {
            InstanceError::NotFound => {
                InstanceHttpError::not_found("instance not found")
            }
            InstanceError::Storage(source) => {
                error!(
                    error = ?source,
                    "instance storage operation failed"
                );
                InstanceHttpError::internal_server_error("failed to load instance")
            }
        }
    }
}

