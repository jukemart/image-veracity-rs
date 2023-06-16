use aide::{
    axum::{routing::post_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::response::Html;
use hex::FromHex;
use tracing::error;

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
        Err(_) => return (StatusCode::BAD_REQUEST, Json(VeracityHash::default())),
    } {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        return if let Ok(hash) = server::stream_to_file(&file_name, field).await {
            match trillian
                .add_leaf(
                    &trillian_tree,
                    hash.crypto_hash.to_string().as_bytes(),
                    hash.perceptual_hash.to_string().as_bytes(),
                )
                .await
            {
                Ok(_) => (StatusCode::CREATED, Json(hash)),
                Err(err) => {
                    error!("Could not add leaf: {}", err);
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(VeracityHash::default()),
                    )
                }
            }
        } else {
            (StatusCode::BAD_REQUEST, Json(VeracityHash::default()))
        };
    }
    (StatusCode::BAD_REQUEST, Json(VeracityHash::default()))
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
}
