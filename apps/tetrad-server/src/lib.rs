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

    let instance_service: InstanceService = instance::create_service(db.clone());

    let current_instance: Instance = instance_service
        .ensure_exists(&config.instance_name)
        .await?;

    info!(
        instance_id = %current_instance.id,
        instance_name = %current_instance.name,
        completed_at = current_instance.setup_completed_at_ms,
        "instance initialized"
    );

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();

    info!("server listening on http://{}", &config.bind_address);

    let state = AppState::new(db, config, instance_service);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(InstanceRouter())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
