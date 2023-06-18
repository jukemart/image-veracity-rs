use aide::axum::IntoApiResponse;
use aide::operation::OperationIo;
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_jsonschema::JsonSchemaRejection;
use axum_macros::FromRequest;
use axum::async_trait;
use axum::http::request::Parts;
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use postgres_openssl::MakeTlsConnector;
use serde::Serialize;
use serde_json::json;

use crate::errors::AppError;
use crate::state::{AppState, ConnectionPool};

#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(AppError))]
#[aide(
    input_with = "axum_jsonschema::Json<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

impl From<JsonSchemaRejection> for AppError {
    fn from(rejection: JsonSchemaRejection) -> Self {
        match rejection {
            JsonSchemaRejection::Json(j) => Self::new(&j.to_string()),
            JsonSchemaRejection::Serde(_) => Self::new("invalid request"),
            JsonSchemaRejection::Schema(s) => {
                Self::new("invalid request").with_details(json!({ "schema_validation": s }))
            }
        }
    }
}
