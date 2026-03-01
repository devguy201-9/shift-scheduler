use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScheduleApiError {
    #[error("Schedule not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ScheduleApiError {
    fn into_response(self) -> Response {
        match self {
            ScheduleApiError::NotFound => StatusCode::NOT_FOUND.into_response(),

            ScheduleApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),

            ScheduleApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}
