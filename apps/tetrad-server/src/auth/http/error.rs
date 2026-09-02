use crate::auth::{AuthError, TetradAuthBackend};

use axum::{
    Json,
    http::{StatusCode, status},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Serialize)]
struct AuthErrorDto {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub(crate) struct AuthHttpError {
    status: StatusCode,
    body: AuthErrorDto,
}

impl AuthHttpError {
    fn internal_server_error(message: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: AuthErrorDto {
                code: "internal_server_error",
                message,
            },
        }
    }

    fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: AuthErrorDto {
                code: "bad_request",
                message,
            },
        }
    }

    fn conflict(message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: AuthErrorDto {
                code: "conflict",
                message,
            },
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: AuthErrorDto {
                code: "not_found",
                message,
            },
        }
    }

    pub(super) fn invalid_credentials() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: AuthErrorDto {
                code: "invalid_credentials",
                message: "invalid username or password",
            },
        }
    }

    pub(super) fn unauthorized(message: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: AuthErrorDto {
                code: "unauthorized",
                message
            }
        }
    }
}

impl IntoResponse for AuthHttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<AuthError> for AuthHttpError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::Internal(source) => {
                error!(
                    error = ?source,
                    "internal authentication operation failed"
                );
                AuthHttpError::internal_server_error("an unexpected error occured")
            }
            AuthError::InvalidUsername => {
                tracing::warn!("user creation rejected due to invalid username");
                AuthHttpError::bad_request("invalid username")
            }
            AuthError::InvalidPassword => {
                //newly selected password fails password rules
                tracing::warn!("user creation rejected due to invalid password");
                AuthHttpError::bad_request("invalid password")
            }
            AuthError::UsernameAlreadyTaken => AuthHttpError::conflict("username is already taken"),
            AuthError::UserNotFound => AuthHttpError::not_found("user not found"),
        }
    }
}

impl From<axum_login::Error<TetradAuthBackend>> for AuthHttpError {
    fn from(error: axum_login::Error<TetradAuthBackend>) -> Self {
        match error {
            axum_login::Error::Backend(error) => error.into(),
            axum_login::Error::Session(error) => {
                error!(
                    error = ?error,
                    "authentication session operation failed"
                );
                AuthHttpError::internal_server_error("an unexpected error occured")
            }
        }
    }
}
