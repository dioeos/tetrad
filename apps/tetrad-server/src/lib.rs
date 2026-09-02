mod auth;
mod common;
mod config;
mod database;
mod instance;
mod state;

use axum::{Router, routing::get};
use axum_login::{
    AuthManagerLayerBuilder, login_required, tower_sessions::{MemoryStore, SessionManagerLayer}
};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::{AuthService, TetradAuthBackend, public_router as auth_public_router, protected_router as auth_protected_router},
    instance::{Instance, InstanceService, router as instance_router},
    state::AppState,
};

pub use config::Config;

pub async fn build_app(db: SqlitePool, config: Config) -> anyhow::Result<Router> {
    let instance_service: InstanceService = instance::create_service(db.clone());
    let auth_service: AuthService = auth::create_service(db.clone());

    let current_instance: Instance = instance_service
        .ensure_exists(&config.instance_name)
        .await?;

    info!(
        id = current_instance.id,
        name = current_instance.name,
        "instance initialized"
    );

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let auth_backend = TetradAuthBackend::new(auth_service.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let state = AppState::new(db, config, instance_service, auth_service);

    let public_api = Router::new()
        .merge(instance_router())
        .merge(auth_public_router());

    let protected_api = Router::new()
        .merge(auth_protected_router())
        .route_layer(login_required!(TetradAuthBackend, login_url = "/login"));

    Ok(Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(public_api)
        .merge(protected_api)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(auth_layer))
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tetrad_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        db_url = &config.database_url,
        bind_address = %config.bind_address,
        instance = &config.instance_name,
        base_url = &config.base_url,
        "tetrad server configurations"
    );

    let db: SqlitePool = database::connect(&config.database_url).await?;
    database::migrate(&db).await?;

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();

    info!("server listening on http://{}", &config.bind_address);

    let app = build_app(db, config).await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
