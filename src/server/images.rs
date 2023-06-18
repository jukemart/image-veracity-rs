use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use hex::FromHex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error};

use crate::errors::AppError;
use crate::extractors::Json;
use crate::hash::cryptographic::CryptographicHash;
use crate::hash::perceptual::PerceptualHash;
use crate::hash::VeracityHash;
use crate::state::AppState;

pub fn image_routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
        .api_route("/:id", get_with(get_image, get_image_docs))
        .with_state(state)
}

async fn get_image(
    State(AppState { db_pool, .. }): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoApiResponse {
    let pool = db_pool.clone();
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            error!("{}", err);
            return db_error().into_response();
        }
    };
    // create the accounts and get the IDs
    let id_hex: [u8; 32] = match <[u8; 32]>::from_hex(&id) {
        Ok(x) => x,
        Err(err) => {
            return AppError::new("Invalid id")
                .with_details(json!(err.to_string()))
                .with_status(StatusCode::BAD_REQUEST)
                .into_response();
        }
    };

    let image_vec: (Vec<u8>, Vec<u8>) = match conn
        .query(
            "SELECT c_hash, p_hash FROM images WHERE c_hash = $1::BYTEA LIMIT 1",
            &[&&id_hex[..]],
        )
        .await
    {
        Ok(result) => match &result[..] {
            [row_hashes] => (row_hashes.get(0), row_hashes.get(1)),
            _ => {
                debug!("No records found for {}", &id);
                return StatusCode::NOT_FOUND.into_response();
            }
        },
        Err(err) => {
            error!("Error getting from database: {}", err);
            return db_error().into_response();
        }
    };

    let image = VeracityHash {
        crypto_hash: CryptographicHash::try_from(image_vec.0).unwrap(),
        perceptual_hash: PerceptualHash::try_from(image_vec.1).unwrap(),
    };
    debug!("retrieved {}", image.crypto_hash);
    Json(image).into_response()
}

fn db_error() -> AppError {
    AppError::new("Could not get image details").with_status(StatusCode::SERVICE_UNAVAILABLE)
}

fn get_image_docs(op: TransformOperation) -> TransformOperation {
    op.description("Get image details")
        .response_with::<200, Json<VeracityHashOutput>, _>(|res| {
            res.example(VeracityHash {
                perceptual_hash: PerceptualHash::from_hex(
                    "9cfde03dc4198467ad671d171c071c5b1ff81bf919d9181838f8f890f807ff01",
                )
                .unwrap(),
                crypto_hash: CryptographicHash::from_b64(
                    "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U",
                )
                .unwrap(),
            })
        })
        .response_with::<400, Json<AppError>, _>(|res| {
            res.description("invalid request")
                .example(AppError::new("Invalid Id").with_status(StatusCode::BAD_REQUEST))
        })
        .response_with::<404, (), _>(|res| res.description("image not found"))
        .response_with::<503, Json<AppError>, _>(|res| {
            res.description("service not available").example(db_error())
        })
}

#[derive(Default, Serialize, Deserialize, JsonSchema)]
pub struct VeracityHashOutput {
    pub crypto_hash: String,
    pub perceptual_hash: String,
}

impl From<VeracityHash> for VeracityHashOutput {
    fn from(value: VeracityHash) -> Self {
        VeracityHashOutput {
            crypto_hash: value.crypto_hash.to_hex(),
            perceptual_hash: value.perceptual_hash.to_hex(),
        }
    }
}
