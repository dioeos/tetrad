use axum::{Json, extract::State};
use tetrad_api_contract::{RegisterDto, RegisterRequest, RegisterUserResponse};
use torii_axum::ConnectionInfo;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{profile::NewProfile, state::AppState};
use crate::error::HttpError;

pub(super) async fn register_handler(
    State(state): State<AppState>,
    connection_info: ConnectionInfo,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterDto>, HttpError> {
    let user = state
        .torii
        .password()
        .register(&request.email, &request.password)
        .await
        .map_err(|error| {
            warn!(?error, "registration failed");
            HttpError::bad_request("registration failed")
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
            HttpError::internal_server_error(
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
            HttpError::internal_server_error(
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
