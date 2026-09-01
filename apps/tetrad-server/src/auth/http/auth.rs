use super::error::AuthHttpError;
use crate::auth::{AuthService, User, model::CreateUserInput};

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct UserDto {
    external_id: String,
    username: String,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            external_id: user.external_id.to_string(),
            username: user.username,
        }
    }
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
}

impl From<CreateUserRequest> for CreateUserInput {
    fn from(request: CreateUserRequest) -> Self {
        Self {
            username: request.username,
            password: request.password,
        }
    }
}

async fn create_user(
    State(auth_service): State<AuthService>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, AuthHttpError> {
    let user = auth_service.create_user(request.into()).await?;
    Ok(Json(user.into()))
}
