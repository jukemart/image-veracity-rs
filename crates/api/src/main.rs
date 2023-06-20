use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{OpenApi, Tag},
    transform::TransformOpenApi,
};
use axum::http::StatusCode;
use axum::Extension;
use eyre::{Report, Result};
use hyper::Method;
use tokio::signal;
use tokio::time::Instant;
use tower_http::cors::{any, Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use image_veracity::state::{AppState, AppStateBuilder};
use image_veracity::{docs::docs_routes, errors::AppError, extractors::Json, server::routes};

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();

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

    let trillian_address = env::var("TRILLIAN_ADDRESS").map_err(|err| {
        error!("Could not get TRILLIAN_ADDRESS: {}", err);
        Report::from(err)
    })?;
    let tree_id = env::var("TRILLIAN_TREE_ID")
        .map_err(|err| {
            error!("Could not get TRILLIAN_TREE_ID: {}", err);
            Report::from(err)
        })?
        .parse::<i64>()
        .map_err(|err| {
            error!("Could not parse TRILLIAN_TREE_ID: {}", err);
            Report::from(err)
        })?;

    let db_connection_uri = env::var("DATABASE_URL")
        .expect("$DATABASE_URL is not set")
        .to_owned();

    let state = AppStateBuilder::default()
        .create_trillian_client(&trillian_address)
        .trillian_tree(tree_id)
        .create_postgres_client(&db_connection_uri)
        .build()
        .await?;
    let mut api = OpenApi::default();

    // Ensure tables at startup as well as db connection works
    create_db_tables(&state).await;

    let cors = CorsLayer::new()
        // allow any methods to access the resource
        .allow_methods(Any)
        // allow requests from any origin
        .allow_origin(Any);

    let app = app(&state)
        .finish_api_with(&mut api, api_docs)
        .layer(cors)
        .layer(Extension(Arc::new(api)))
        .with_state(state);

    // send it
    let addr = if let Ok(addr) = env::var("LISTEN_ADDRESS") {
        addr.parse()?
    } else {
        SocketAddr::from(([127, 0, 0, 1], 3000))
    };
    debug!("Listening on {}", addr);
    let startup_duration = start.elapsed();
    info!("Startup time: {:?}", startup_duration);
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

fn app(state: &AppState) -> ApiRouter<AppState> {
    ApiRouter::new()
        .nest_api_service("/", routes::server_routes(state.clone()))
        .nest_api_service("/docs", docs_routes(state.clone()))
        // We can still add middleware
        .layer(TraceLayer::new_for_http())
}

async fn create_db_tables(state: &AppState) {
    let pool = &state.db_pool.clone();
    let conn = pool.get().await.expect("database connection");
    // Create the "images" table.
    match conn
        .execute(
            "CREATE TABLE IF NOT EXISTS images (c_hash BYTES NOT NULL PRIMARY KEY, p_hash BYTES NOT NULL)",
            &[],
        )
        .await {
        Ok(result) => {
            info!("Create table result {}", result);
        }
        Err(err) => error!("{}", err)
    };
    match conn
        .execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS images_p_hash_index ON images (p_hash)",
            &[],
        )
        .await
    {
        Ok(result) => {
            info!("Create p_hash index result {}", result);
        }
        Err(err) => error!("{}", err),
    }
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
