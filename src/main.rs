use std::io;
use std::net::SocketAddr;

use axum::{
    body::Bytes,
    BoxError,
    extract::{BodyStream, Multipart, Path},
    http::StatusCode,
    response::{Html, Redirect},
    Router,
    routing::{get, post},
};
use axum::extract::DefaultBodyLimit;
use futures::{Stream, TryStreamExt};
use tokio::{io::ErrorKind};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::{info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use image_veracity;

const UPLOADS_DIRECTORY: &str = "uploads";
const MAX_UPLOAD_SIZE: usize = 1024 * 1024 * 5;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "image_veracity=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Save files to separate directory to not override files in current directory
    match tokio::fs::create_dir(UPLOADS_DIRECTORY).await {
        Ok(_) => info!("{} directory created", UPLOADS_DIRECTORY),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            info!("{} directory already exists", UPLOADS_DIRECTORY)
        }
        Err(err) => panic!("could not create directory {}: {}", UPLOADS_DIRECTORY, err.to_string())
    }

    // build app with route
    let app = Router::new()
        .route("/", get(show_form).post(accept_form))
        .route("/file/:file_name", post(save_request_body))
        // Set max file size to 5MB
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE));

    // send it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html lang="en">
            <head>
                <title>Upload something!</title>
            </head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <div>
                        <label>
                            Upload file:
                            <input type="file" name="file" multiple>
                        </label>
                    </div>

                    <div>
                        <input type="submit" value="Upload files">
                    </div>
                </form>
            </body>
        </html>
        "#
    )
}

async fn accept_form(mut multipart: Multipart) -> Result<Redirect, (StatusCode, String)> {
    while let Some(field) = match multipart.next_field().await {
        Ok(x) => x,
        Err(err) => return Err((StatusCode::BAD_REQUEST, err.to_string())),
    } {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        stream_to_file(&file_name, field).await?;
    }

    Ok(Redirect::to("/"))
}

async fn save_request_body(
    Path(file_name): Path<String>,
    body: BodyStream,
) -> Result<(), (StatusCode, String)> {
    stream_to_file(&file_name, body).await
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)> where S: Stream<Item=Result<Bytes, E>>, E: Into<BoxError>, {
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert stream into AsyncRead
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);

        info!("created stream reader");

        futures::pin_mut!(body_reader);

        // Create file. File implements AsyncWrite
        // let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        // let mut file = BufWriter::new(File::create(path).await?);
        //
        // // Copy body into file
        // tokio::io::copy(&mut body_reader, &mut file).await?;

        // info!("copied to file");

        let mut buffer = Vec::new();
        body_reader.read_to_end(&mut buffer).await?;
        if let Ok((phash, chash)) = image_veracity::hash::hash_image(&buffer) {
            info!("image phash {} chash {}", phash, chash);
        }

        Ok::<_, io::Error>(())
    }
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
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
