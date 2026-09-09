mod config;
mod error;
mod state;

mod entities;
mod model;

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, Response},
    routing::get,
};
use tower_http::trace::TraceLayer;
use tracing::{Span, debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    entities::instance_routes,
    model::{InstanceBmc, InstanceForCreate, ModelManager},
    state::AppState,
};

pub use config::{Config, use_config};

pub async fn build_app(database_url: &str, instance_name: &str) -> anyhow::Result<Router> {
    // let current_instance: Instance = instance_service
    //     .ensure_exists(&config.instance_name)
    //     .await?;

    // info!(
    //     id = current_instance.id,
    //     name = current_instance.name,
    //     "instance initialized"
    // );
    let model_manager = ModelManager::new(database_url).await?;

    let instance_c = InstanceForCreate {
        name: instance_name.to_owned(),
    };

    let _ = InstanceBmc::ensure_exists(&model_manager, instance_c).await?;

    // let state = AppState::new(model_manager);

    Ok(Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(instance_routes(model_manager.clone()))
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

pub async fn run(config: &'static Config) -> anyhow::Result<()> {
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

    let app = build_app(&config.database_url, &config.instance_name).await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
