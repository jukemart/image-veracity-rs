pub mod routes;

use crate::hash::{hash_image, VeracityHash};
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::BoxError;
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::{debug, error};

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<VeracityHash, (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
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
                return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
            }
        }

        let veracity = parallel_hash(buffer).await;
        Ok(veracity)
    }
    .await
}

async fn parallel_hash(buffer: Vec<u8>) -> VeracityHash {
    let (send, recv) = tokio::sync::oneshot::channel();

    // Spawn a task on rayon.
    rayon::spawn(move || {
        if let Ok(veracity) = hash_image(&buffer) {
            debug!(
                "image phash {} chash {}",
                veracity.perceptual_hash, veracity.crypto_hash
            );

            // Send the result back to Tokio.
            let _ = send.send(veracity);
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
