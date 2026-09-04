mod common;
mod config;
mod database;
mod instance;
mod profile;
mod state;
mod torii_tetrad;

use std::sync::Arc;
use std::time::Duration;

use axum::{Router, body::Body, http::{Request, Response}, routing::get};
use sqlx::SqlitePool;
use torii::Torii;
use torii_storage_seaorm::SeaORMStorage;
use tower_http::trace::TraceLayer;
use tracing::{Span, debug, info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    instance::{Instance, InstanceService, router as instance_router},
    profile::ProfileService,
    state::AppState,
    torii_tetrad::custom_torii_auth_router,
};

pub use config::Config;

async fn initialize_database(database_url: &str) -> anyhow::Result<(SqlitePool, SeaORMStorage)> {
    let db: SqlitePool = database::connect(database_url).await?;

    let storage = SeaORMStorage::connect(database_url).await?;
    storage.migrate().await?;

    database::migrate(&db).await?;

    Ok((db, storage))
}

pub async fn build_app(config: Config) -> anyhow::Result<Router> {
    let (db, storage) = initialize_database(&config.database_url).await?;
    let instance_service: InstanceService = instance::create_service(db.clone());
    let profile_service: ProfileService = profile::create_service(db.clone());

    let current_instance: Instance = instance_service
        .ensure_exists(&config.instance_name)
        .await?;

    info!(
        id = current_instance.id,
        name = current_instance.name,
        "instance initialized"
    );

    let repos = Arc::new(storage.into_repository_provider());
    let torii = Arc::new(Torii::new(repos));

    let state = AppState::new(db, config, torii, instance_service, profile_service);

    let auth_routes = Router::new().merge(custom_torii_auth_router());

    Ok(Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(instance_router())
        .nest("/auth", auth_routes)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<Body>, _span: &Span| {
                    debug!(
                        method = %request.method(),
                        uri = %request.uri(),
                        "request started"
                    );
                })
                .on_response(
                    |response: &Response<Body>, latency: Duration, _span: &Span| {
                        let status = response.status();

                        if status.is_server_error() {
                            error!(
                                status = %status,
                                latency_ms = latency.as_millis(),
                                "request failed"
                            );
                        } else if status.is_client_error() {
                            warn!(
                                status = %status,
                                latency_ms = latency.as_millis(),
                                "request rejected"
                            );
                        } else {
                            info!(
                                status = %status,
                                latency_ms = latency.as_millis(),
                                "request completed"
                            );
                        }
                    },
                ),
        ))
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tetrad_server=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_target(false),
        )
        .init();

    info!(
        db_url = &config.database_url,
        bind_address = %config.bind_address,
        instance = &config.instance_name,
        base_url = &config.base_url,
        "tetrad server configurations"
    );

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();

    info!("server listening on http://{}", &config.bind_address);

    let app = build_app(config).await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
