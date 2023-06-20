use std::io;

use axum::body::Bytes;
use axum::BoxError;
use futures::{Stream, TryStreamExt};
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::{debug, error};

use crate::errors::AppError;
use crate::hash::{hash_image, HashError, VeracityHash};

mod images;
pub mod routes;

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<VeracityHash, AppError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err(AppError::new("Invalid path"));
    }

    async {
        // Convert stream into AsyncRead
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);

        debug!("created stream reader");

        futures::pin_mut!(body_reader);

        let mut buffer = Vec::new();
        match body_reader.read_to_end(&mut buffer).await {
            Ok(_) => debug!("read multipart buffer"),
            Err(err) => {
                error!("could not read buffer: {}", err.to_string());
                return Err(AppError::new("could not read file to buffer")
                    .with_details(json!(err.to_string())));
            }
        }

        match parallel_hash(buffer).await {
            Ok(hash) => {
                debug!("created hash {:?}", hash);
                Ok(hash)
            }
            Err(err) => {
                error!("error while hashing {}", err.to_string());
                Err(AppError::new(&err.to_string()))
            }
        }
    }
    .await
}

async fn parallel_hash(buffer: Vec<u8>) -> Result<VeracityHash, HashError> {
    let (send, recv) = tokio::sync::oneshot::channel();

    // Spawn a task on rayon.
    rayon::spawn(move || {
        match hash_image(&buffer) {
            Ok(veracity) => {
                debug!(
                    "image phash {} chash {}",
                    veracity.perceptual_hash, veracity.crypto_hash
                );
                // Send the result back to Tokio.
                let _ = send.send(Ok(veracity));
            }
            Err(err) => {
                error!("{}", err);
                let _ = send.send(Err(err));
            }
        }
    });

    // Wait for the rayon task.
    recv.await.expect("Panic in rayon::spawn")
}

fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
