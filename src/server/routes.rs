use aide::{
    axum::{routing::post_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use hex::FromHex;
use trillian::client::TrillianClient;

use crate::errors::AppError;
use crate::hash::{cryptographic::CryptographicHash, perceptual::PerceptualHash, VeracityHash};
use crate::server;
use crate::{extractors::Json, state::AppState};

const MAX_UPLOAD_SIZE: usize = 1024 * 1024 * 5;

pub fn server_routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
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
    op.description("Upload form an image to get veracity hashes")
        .response_with::<200, (), _>(|res| res.description("Form upload HTML"))
}

async fn accept_form(
    State(AppState {
        mut trillian,
        trillian_tree,
        ..
    }): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoApiResponse {
    while let Some(field) = match multipart.next_field().await {
        Ok(x) => x,
        Err(err) => {
            return AppError::new(&err.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .into_response()
        }
    } {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        return match server::stream_to_file(&file_name, field).await {
            Ok(hash) => return add_hash_to_tree(&mut trillian, &trillian_tree, hash).await,
            Err(err) => AppError::new(err.1.as_str())
                .with_status(err.0)
                .into_response(),
        };
    }
    AppError::new("no multipart fields found")
        .with_status(StatusCode::BAD_REQUEST)
        .into_response()
}

async fn add_hash_to_tree(
    trillian: &mut TrillianClient,
    trillian_tree: &i64,
    hash: VeracityHash,
) -> Response {
    match trillian
        .add_leaf(
            trillian_tree,
            hash.crypto_hash.as_ref(),
            hash.perceptual_hash.as_ref(),
        )
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(hash)).into_response(),
        Err(err) => AppError::new(&err.to_string())
            .with_status(StatusCode::SERVICE_UNAVAILABLE)
            .into_response(),
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
        .response_with::<400, (), _>(|res| res.description("could not process request"))
        .response_with::<503, (), _>(|res| res.description("downstream dependency unavailable"))
}
