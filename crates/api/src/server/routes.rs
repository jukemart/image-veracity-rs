use aide::axum::routing::get_with;
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
use tracing::log::debug;
use tracing::{error, warn};

use trillian::TrillianLogLeaf;

use crate::errors::AppError;
use crate::hash::{cryptographic::CryptographicHash, perceptual::PerceptualHash, VeracityHash};
use crate::server::images;
use crate::state::TrillianState;
use crate::{extractors::Json, server, state::AppState};

const MAX_UPLOAD_SIZE: usize = 1024 * 1024 * 20;

pub fn server_routes(state: AppState) -> ApiRouter {
    app(&state).nest_api_service("/images", images::image_routes(state))
}

fn app(state: &AppState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            post_with(accept_form, accept_form_docs).get_with(show_form, show_form_docs),
        )
        .api_route("/healthcheck", get_with(healthcheck, healthcheck_docs))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE))
        .with_state(state.clone())
}

async fn healthcheck(State(AppState { db_pool, .. }): State<AppState>) -> impl IntoApiResponse {
    let pool = db_pool.clone();
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            error!("{}", err);
            return db_error().into_response();
        }
    };

    match conn.query("SELECT 1", &[]).await {
        Ok(_) => (StatusCode::OK, "healthy").into_response(),
        Err(_) => db_error().into_response(),
    }
}

fn healthcheck_docs(op: TransformOperation) -> TransformOperation {
    op.description("Healthcheck")
        .response_with::<200, (), _>(|res| res.description("Application is healthy"))
        .response_with::<503, (), _>(|res| res.description("Application is unhealthy"))
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
        trillian,
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

        let (hash, _leaf) = match add_hash_to_tree(trillian, &trillian_tree, hash).await {
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
        match conn
            .query(
                "INSERT INTO images (c_hash, p_hash) VALUES ($1, $2)",
                &[
                    &hash.crypto_hash.as_ref().to_vec(),
                    &hash.perceptual_hash.as_ref().to_vec(),
                ],
            )
            .await
        {
            Ok(_) => {}
            Err(err) => {
                warn!("Could not add to database: {}", err.to_string());
                return if err.to_string().contains("duplicate") {
                    AppError::new("image already exists in database")
                        .with_status(StatusCode::CONFLICT)
                        .into_response()
                } else {
                    db_error().into_response()
                };
            }
        };

        debug!(
            "added c_hash {} p_hash {}",
            &hash.crypto_hash, &hash.perceptual_hash
        );
        let mut res = Json(hash).into_response();
        *res.status_mut() = StatusCode::CREATED;
        return res;
    }
    AppError::new("no multipart fields found")
        .with_status(StatusCode::BAD_REQUEST)
        .into_response()
}

async fn add_hash_to_tree(
    mut trillian: TrillianState,
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

#[cfg(test)]
mod tests {
    use std::net::{SocketAddr, TcpListener};

    use aide::openapi::OpenApi;
    use async_trait::async_trait;
    use axum::{body::Body, http::Request};
    use hyper::Method;
    use mockall::mock;

    use trillian::client::TrillianClientApiMethods;
    use trillian::{TrillianLogLeaf, TrillianTree};

    use crate::state::AppStateBuilder;

    use super::*;

    mock! {
        pub TrillianClient {
          fn get_leaf(&self) -> TrillianLogLeaf {
            TrillianLogLeaf::default()
        }
        fn get_tree(&self) -> TrillianTree {
            TrillianTree::default()
        }
      }
    }

    #[async_trait]
    impl TrillianClientApiMethods for MockTrillianClient {
        async fn add_leaf(
            &mut self,
            _id: &i64,
            _data: &[u8],
            _extra_data: &[u8],
        ) -> Result<TrillianLogLeaf> {
            Ok(self.get_leaf())
        }
        async fn create_tree(&mut self, _name: &str, _description: &str) -> Result<TrillianTree> {
            Ok(self.get_tree())
        }
        async fn list_trees(&mut self) -> Result<Vec<TrillianTree>> {
            Ok(vec![self.get_tree()])
        }
    }

    impl Clone for MockTrillianClient {
        fn clone(&self) -> Self {
            MockTrillianClient::new()
        }
    }

    async fn mock_state() -> AppState {
        // TODO mock this as well
        let database_url = "postgresql://root@localhost:26257/veracity?sslmode=disable";
        AppStateBuilder::default()
            .trillian(Box::from(MockTrillianClient::new()))
            .trillian_host("http://localhost:8090".to_string())
            .trillian_tree(0)
            .create_postgres_client(database_url)
            .build()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn get_show_form() {
        let addr = start_test_server().await;

        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("http://{}/", addr))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn does_not_exist() {
        let addr = start_test_server().await;

        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("http://{}/does-not-exist", addr))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    async fn start_test_server() -> SocketAddr {
        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        let state = mock_state().await;

        tokio::spawn(async move {
            let mut api = OpenApi::default();
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app(&state).finish_api(&mut api).into_make_service())
                .await
                .unwrap();
        });
        addr
    }
}
