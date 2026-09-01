mod config;
mod database;
mod instance;
mod state;

use axum::{Router, routing::get};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    instance::{Instance, InstanceService, router as InstanceRouter},
    state::AppState,
};

pub use config::Config;

pub async fn build_app(db: SqlitePool, config: Config) -> anyhow::Result<Router> {
    let instance_service: InstanceService = instance::create_service(db.clone());
    let current_instance: Instance = instance_service
        .ensure_exists(&config.instance_name)
        .await?;

    info!(
        id = current_instance.id,
        name = current_instance.name,
        "instance initialized"
    );

    let state = AppState::new(db, config, instance_service);

    Ok(Router::new()
        .route("/", get(|| async { "Hellow, World!" }))
        .merge(InstanceRouter())
        .with_state(state)
        .layer(TraceLayer::new_for_http()))
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tetrad_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
