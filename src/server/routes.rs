use crate::hash::VeracityHash;
use crate::server;
use crate::{extractors::Json, state::AppState};
use aide::{
    axum::{routing::post_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::http::StatusCode;
use axum::response::Html;

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

async fn accept_form(mut multipart: Multipart) -> impl IntoApiResponse {
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
            (StatusCode::CREATED, Json(hash))
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
                perceptual_hash: "9cfde03dc4198467ad671d171c071c5b1ff81bf919d9181838f8f890f807ff01"
                    .to_string(),
                crypto_hash: "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U".to_string(),
            })
        })
        .response_with::<400, (), _>(|res| res.description("could not process request"))
}
