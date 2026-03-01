use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("database")]
    Database(#[source] sqlx::Error),

    #[error("cache")]
    Cache(#[source] redis::RedisError),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &e {
            if let Some(code) = db_err.code() {
                match code.as_ref() {
                    // unique_violation
                    "23505" => {
                        return AppError::Conflict(
                            "Duplicate value violates unique constraint".into(),
                        );
                    }
                    // foreign_key_violation
                    "23503" => {
                        return AppError::Conflict("Foreign key constraint violation".into());
                    }
                    _ => {}
                }
            }
        }

        AppError::Database(e)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        AppError::Cache(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg).into_response(),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
            AppError::Cache(e) => {
                tracing::error!(error = %e, "cache error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
        }
    }
}
