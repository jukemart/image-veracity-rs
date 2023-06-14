use std::net::SocketAddr;
use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{OpenApi, Tag},
    transform::TransformOpenApi,
};
use axum::http::StatusCode;
use axum::Extension;
use eyre::Result;
use tokio::{io::ErrorKind, signal};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use image_veracity::{
    docs::docs_routes, errors::AppError, extractors::Json, server::routes, state::AppState,
};

const UPLOADS_DIRECTORY: &str = "uploads";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "image_veracity=debug,trillian_client=debug,hyper=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    aide::gen::on_error(|error| {
        error!("{error}");
    });

    aide::gen::extract_schemas(true);

    // TODO replace with ENV var
    let state = AppState::new("http://localhost:8090".to_string()).await?;
    let mut api = OpenApi::default();

    // Save files to separate directory to not override files in current directory
    match tokio::fs::create_dir(UPLOADS_DIRECTORY).await {
        Ok(_) => info!("Directory `{}` created", UPLOADS_DIRECTORY),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            info!("Directory `{}` already exists", UPLOADS_DIRECTORY)
        }
        Err(err) => panic!(
            "could not create directory {}: {}",
            UPLOADS_DIRECTORY,
            err.to_string()
        ),
    }

    let app = ApiRouter::new()
        .nest_api_service("/", routes::server_routes(state.clone()))
        .nest_api_service("/docs", docs_routes(state.clone()))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .with_state(state);

    info!("Documentation accessible at http://127.0.0.1:3000/docs");

    // send it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("Listening on {}", addr);
    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        Ok(_) => info!("Server shut down successfully"),
        Err(e) => error!("Could not shutdown server: {}", e.to_string()),
    };
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Veracity Image Open API")
        .summary("Datastore for verified images")
        .description(include_str!("README.md"))
        .tag(Tag {
            name: "image".into(),
            description: Some("Image Veracity".into()),
            ..Default::default()
        })
        .security_scheme(
            "ApiKey",
            aide::openapi::SecurityScheme::ApiKey {
                location: aide::openapi::ApiKeyLocation::Header,
                name: "X-Auth-Key".into(),
                description: Some("A key that is ignored.".into()),
                extensions: Default::default(),
            },
        )
        .default_response_with::<Json<AppError>, _>(|res| {
            res.example(AppError {
                error: "some error happened".to_string(),
                error_details: None,
                error_id: Uuid::nil(),
                // This is not visible.
                status: StatusCode::IM_A_TEAPOT,
            })
        })
}
