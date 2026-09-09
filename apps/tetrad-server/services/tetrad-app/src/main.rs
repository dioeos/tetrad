mod error;
mod routes;

use axum::{Router, routing::get};
use error::Error;

use tetrad_core::{config, model::{ModelManager, instance::{InstanceBmc, InstanceForCreate}}};
use tracing::{info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::use_config();

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
        "{:<12} - DB: {}, Bind: {}, Instance: {}, Base: {}",
        "TETRAD SERVER CONFIG",
        config.database_url,
        config.bind_address,
        config.instance_name,
        config.base_url,
    );

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();

    info!("{:<12} - {}", "LISTENING", &config.bind_address);

    let model_manager = ModelManager::new(&config.database_url).await?;

    let instance_c = InstanceForCreate {
        name: config::use_config().instance_name.to_owned()
    };

    let _ = InstanceBmc::ensure_exists(&model_manager, instance_c).await?;

    let all_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(routes::instance_routes::routes(model_manager.clone()));

    axum::serve(listener, all_routes.into_make_service()).await.unwrap();

    Ok(())
}
