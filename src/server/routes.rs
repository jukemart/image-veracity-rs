use aide::{
    axum::{routing::post_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use eyre::Result;
use hex::FromHex;
use serde_json::json;
use tracing::{error, info};

use trillian::client::TrillianClient;
use trillian::TrillianLogLeaf;

use crate::errors::AppError;
use crate::hash::{cryptographic::CryptographicHash, perceptual::PerceptualHash, VeracityHash};
use crate::server;
use crate::server::images;
use crate::{extractors::Json, state::AppState};

const MAX_UPLOAD_SIZE: usize = 1024 * 1024 * 5;

pub fn server_routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
        .nest_api_service("/images", images::image_routes(state.clone()))
        .api_route(
            "/",
            post_with(accept_form, accept_form_docs).get_with(show_form, show_form_docs),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE))
        .with_state(state)
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html lang="en">
            <head>
                <title>Image Upload</title>
            </head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <div>
                        <label>
                            Image File:
                            <input type="file" name="image" required>
                        </label>
                    </div>

                    <div>
                        <input type="submit" value="Upload">
                    </div>
                </form>
            </body>
        </html>
        "#,
    )
}

fn show_form_docs(op: TransformOperation) -> TransformOperation {
    op.description("Basic image upload form")
        .response_with::<200, (), _>(|res| res.description("Form upload HTML"))
}

async fn accept_form(
    State(AppState {
        mut trillian,
        trillian_tree,
        db_pool,
        ..
    }): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoApiResponse {
    while let Some(field) = match multipart.next_field().await {
        Ok(x) => x,
        Err(err) => {
            error!("{}", err);
            return AppError::new(&err.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .into_response();
        }
    } {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        let hash = match server::stream_to_file(&file_name, field).await {
            Ok(x) => x,
            Err(err) => {
                return AppError::new("Could not hash image")
                    .with_details(json!(err))
                    .with_status(StatusCode::BAD_REQUEST)
                    .into_response();
            }
        };

        let (hash, leaf) = match add_hash_to_tree(&mut trillian, &trillian_tree, hash).await {
            Ok(x) => x,
            Err(err) => {
                error!("{}", err);
                return AppError::new("Could not add image to Trillian")
                    .with_status(StatusCode::SERVICE_UNAVAILABLE)
                    .into_response();
            }
        };

        // Add leaf to DB
        let pool = db_pool.clone();
        let conn = match pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("{}", err);
                return db_error().into_response();
            }
        };

        // create the accounts and get the IDs
        let hashes: (Vec<u8>, Vec<u8>) = match conn
            .query(
                "INSERT INTO images (c_hash, p_hash) VALUES ($1, $2) RETURNING c_hash, p_hash",
                &[
                    &hash.crypto_hash.as_ref().to_vec(),
                    &hash.perceptual_hash.as_ref().to_vec(),
                ],
            )
            .await
        {
            Ok(result) => match &result[..] {
                [row_hashes] => (row_hashes.get(0), row_hashes.get(1)),
                _ => unreachable!(),
            },
            Err(err) => {
                error!("Could not add to database: {}", err.to_string());
                return if err.to_string().contains("duplicate") {
                    AppError::new("image already exists in database")
                        .with_status(StatusCode::CONFLICT)
                        .into_response()
                } else {
                    db_error().into_response()
                };
            }
        };

        info!("ids {:?}", hashes);
        let mut res = Json(hash).into_response();
        *res.status_mut() = StatusCode::CREATED;
        return res;
    }
    AppError::new("no multipart fields found")
        .with_status(StatusCode::BAD_REQUEST)
        .into_response()
}

async fn add_hash_to_tree(
    trillian: &mut TrillianClient,
    trillian_tree: &i64,
    hash: VeracityHash,
) -> Result<(VeracityHash, TrillianLogLeaf)> {
    match trillian
        .add_leaf(
            trillian_tree,
            hash.crypto_hash.as_ref(),
            hash.perceptual_hash.as_ref(),
        )
        .await
    {
        Ok(leaf) => Ok((hash, leaf)),
        Err(err) => Err(err),
    }
}

fn accept_form_docs(op: TransformOperation) -> TransformOperation {
    op.description("Return a veracity hash")
        .response_with::<201, Json<VeracityHash>, _>(|res| {
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
            res.description("could not process request")
                .example(AppError::new("Could not hash image").with_status(StatusCode::BAD_REQUEST))
        })
        .response_with::<503, Json<AppError>, _>(|res| {
            res.description("downstream dependency unavailable")
                .example(db_error())
        })
}

fn db_error() -> AppError {
    AppError::new("Could add image").with_status(StatusCode::SERVICE_UNAVAILABLE)
}
