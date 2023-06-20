use axum::{http::StatusCode, response::IntoResponse};
use eyre::Report;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

/// A default error response for most API errors.
#[derive(Debug, Error, Serialize, JsonSchema)]
pub struct AppError {
    /// An error message.
    pub error: String,
    /// A unique error ID.
    pub error_id: Uuid,
    #[serde(skip)]
    pub status: StatusCode,
    /// Optional Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Value>,
}
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} id:{} {} {:?}",
            self.status, self.error_id, self.error, self.error_details
        )
    }
}

impl AppError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            error_id: Uuid::new_v4(),
            status: StatusCode::BAD_REQUEST,
            error_details: None,
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.error_details = Some(details);
        self
    }
}

impl IntoResponse for AppError {
    #[instrument]
    fn into_response(self) -> axum::response::Response {
        error!("");
        let status = self.status;
        let mut res = axum::Json(self).into_response();
        *res.status_mut() = status;
        res
    }
}

impl From<Report> for AppError {
    fn from(value: Report) -> Self {
        AppError::new(&value.to_string())
    }
}
