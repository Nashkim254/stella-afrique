use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),
    #[error("resource not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("multipart error")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("storage is not configured")]
    StorageNotConfigured,
    #[error("storage error")]
    Storage,
    #[error("payment provider is not configured")]
    PaymentNotConfigured,
    #[error("external service error: {0}")]
    ExternalService(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Database(error) => {
                error!(error = %error, "database error response");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(message) => {
                error!(error = %message, "bad request response");
                StatusCode::BAD_REQUEST
            }
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Multipart(error) => {
                error!(error = %error, "multipart error response");
                StatusCode::BAD_REQUEST
            }
            Self::StorageNotConfigured | Self::Storage | Self::PaymentNotConfigured => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExternalService(message) => {
                error!(error = %message, "external service error response");
                StatusCode::BAD_GATEWAY
            }
        };

        let body = Json(ErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}
