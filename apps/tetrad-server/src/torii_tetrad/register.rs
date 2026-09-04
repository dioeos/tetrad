use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use torii_axum::ConnectionInfo;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{profile::NewProfile, state::AppState};

#[derive(Deserialize)]
pub(super) struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterUserResponse {
    torii_user_id: String,
    external_id: String,
    email: String,
}

#[derive(Serialize)]
pub(super) struct RegisterDto {
    token: String,
    user: RegisterUserResponse,
}

#[derive(Debug, Serialize)]
pub(super) struct RegisterErrorDto {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub(super) struct RegisterHttpError {
    status: StatusCode,
    body: RegisterErrorDto,
}

impl IntoResponse for RegisterHttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl RegisterHttpError {
    fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: RegisterErrorDto {
                code: "bad_request",
                message,
            },
        }
    }

    fn internal_server_error(message: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: RegisterErrorDto {
                code: "internal_server_error",
                message,
            },
        }
    }
}

pub(super) async fn register_handler(
    State(state): State<AppState>,
    connection_info: ConnectionInfo,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterDto>, RegisterHttpError> {
    let user = state
        .torii
        .password()
        .register(&request.email, &request.password)
        .await
        .map_err(|error| {
            warn!(?error, "registration failed");
            RegisterHttpError::bad_request("registration failed")
        })?;

    let external_id = Uuid::now_v7();
    let torii_user_id = user.id.clone();
    let new_profile = NewProfile {
        torii_user_id: torii_user_id.clone(),
        external_id,
    };

    let profile = state
        .profile_service
        .create_profile(new_profile)
        .await
        .map_err(|error| {
            error!(?error, "profile creation failed");
            RegisterHttpError::internal_server_error(
                "an unexpected error occurred creating profile",
            )
        })?;

    let (user, session) = state
        .torii
        .password()
        .authenticate(
            &request.email,
            &request.password,
            connection_info.user_agent,
            connection_info.ip,
        )
        .await
        .map_err(|error| {
            error!(?error, "session creation failed");
            RegisterHttpError::internal_server_error(
                "an unexpected error occurred creating session",
            )
        })?;

    let token = session.token.as_str().to_owned();

    Ok(Json(RegisterDto {
        token,
        user: RegisterUserResponse {
            torii_user_id: profile.torii_user_id.to_string(),
            external_id: profile.external_id.to_string(),
            email: user.email,
        },
    }))
}
