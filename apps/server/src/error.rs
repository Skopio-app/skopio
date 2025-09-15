use anyhow::anyhow;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use common::time::TimeError;
use db::error::DBError;
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};

/// This type represents all possible errors that can occur when interacting
/// with the server app
#[derive(Debug, Error)]
pub enum ServerError {
    // 4xx
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[allow(dead_code)]
    #[error("Not found")]
    NotFound,

    #[allow(dead_code)]
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unsupported: {0}")]
    Unprocessable(String), // 422

    // 5xx
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

impl ServerError {
    fn status(&self) -> StatusCode {
        match self {
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::NotFound => StatusCode::NOT_FOUND,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn code(&self) -> &'static str {
        match self {
            ServerError::BadRequest(_) => "BAD_REQUEST",
            ServerError::NotFound => "NOT_FOUND",
            ServerError::Conflict(_) => "CONFLICT",
            ServerError::Unprocessable(_) => "UNPROCESSABLE",
            ServerError::Internal(_) => "INTERNAL",
        }
    }
}

impl From<DBError> for ServerError {
    fn from(e: DBError) -> Self {
        match e {
            DBError::MissingField(name) => {
                ServerError::BadRequest(format!("Missing field: {name}"))
            }
            DBError::Parse(err) => Self::BadRequest(err.to_string()),
            DBError::Uuid(err) => ServerError::BadRequest(err.to_string()),
            DBError::Unsupported(msg) => ServerError::Unprocessable(msg.to_string()),
            DBError::Sqlx(err) => ServerError::Internal(anyhow!(err.to_string())),
            DBError::Migration(err) => Self::Internal(anyhow!(err.to_string())),
        }
    }
}

impl From<TimeError> for ServerError {
    fn from(e: TimeError) -> Self {
        match e {
            TimeError::InvalidDate => {
                ServerError::BadRequest("Invalid datetime passed".to_string())
            }
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match &self {
            ServerError::Internal(err) => error!("Internal error: {err:?}"),
            _ => warn!("Client error: {self}"),
        }
        let status = self.status();
        let body = ErrorBody {
            code: self.code(),
            message: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, ServerError>;
