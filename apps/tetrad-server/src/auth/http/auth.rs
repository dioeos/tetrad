use super::error::AuthHttpError;
use crate::auth::{
    AuthService, TetradAuthBackend, User,
    model::{CreateUserInput, Credentials},
};

use axum::{Form, Json, extract::{Path, State}, http::StatusCode};
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
pub(crate) struct CreateUserRequest {
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

pub(crate) async fn create_user(
    State(auth_service): State<AuthService>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, AuthHttpError> {
    let user = auth_service.create_user(request.into()).await?;
    Ok(Json(user.into()))
}

type AuthSession = axum_login::AuthSession<TetradAuthBackend>;

pub(crate) async fn login(mut auth_session: AuthSession, Form(creds): Form<Credentials>) -> Result<StatusCode, AuthHttpError> {
    //@NOTE: The error seen by the handler is not of type `AuthError`,
    //       but of type `axum_login::Error<TetradAuthBackend>, with its errors being
    //       wrapped as `axum_login::Error::Backend(auth_error)` or
    //       `axum_login::Error::Session(auth_error) and being converted. Each of 
    //       these `axum_login::Error` variations are converted into an `AuthHttpError`
    let user = auth_session
        .authenticate(creds)
        .await?
        .ok_or_else(AuthHttpError::invalid_credentials)?;

    auth_session.login(&user).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn me(auth_session: AuthSession) -> Result<Json<UserDto>, AuthHttpError> {
    let current_user: User = auth_session
        .user
        .ok_or_else(|| AuthHttpError::unauthorized("authentication required"))?;

    Ok(Json(current_user.into()))
}

pub(crate) async fn get_user_by_username(
    State(auth_service): State<AuthService>,
    Path(username): Path<String>
) -> Result<Json<UserDto>, AuthHttpError> {
    let user = auth_service
        .get_user_by_username(&username)
        .await?
        .ok_or_else(|| AuthHttpError::not_found("user not found"))?;
    Ok(Json(user.into())) 
}
