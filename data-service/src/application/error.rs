use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Entity not found")]
    NotFound,

    #[error("Conflict")]
    Conflict,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error")]
    Database,

    #[error("Cache error")]
    Cache,
}

impl From<sqlx::Error> for AppError {
    fn from(_: sqlx::Error) -> Self {
        AppError::Database
    }
}

impl From<redis::RedisError> for AppError {
    fn from(_: redis::RedisError) -> Self {
        AppError::Cache
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Conflict => StatusCode::CONFLICT.into_response(),
            AppError::Validation(_) => StatusCode::BAD_REQUEST.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
