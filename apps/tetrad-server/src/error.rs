use axum::{Json, http::StatusCode, response::IntoResponse};
use tetrad_api_contract::ApiErrorResponse;

#[derive(Debug)]
pub(crate) struct HttpError {
    status: StatusCode,
    body: ApiErrorResponse,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl HttpError {
    pub(crate) fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorResponse {
                code: "bad_request".to_owned(),
                message: message.into(),
            },
        }
    }

    pub(crate) fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ApiErrorResponse {
                code: "not_found".to_owned(),
                message: message.into(),
            },
        }
    }

    pub(crate) fn internal_server_error(message: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiErrorResponse {
                code: "internal_server_error".to_owned(),
                message: message.into(),
            },
        }
    }
}
