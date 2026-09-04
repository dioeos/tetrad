mod common;
mod config;
mod database;
mod error;
mod instance;
mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, Response},
    routing::get,
};
use torii::Torii;
use torii_axum::{CookieConfig, LinkConfig};
use tower_http::trace::TraceLayer;
use tracing::{Span, debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    instance::{Instance, InstanceService, router as instance_router},
    state::AppState,
};

pub use config::Config;

pub async fn build_app(config: Config) -> anyhow::Result<Router> {
    let (db, storage) = database::initialize(&config.database_url).await?;
    let instance_service: InstanceService = instance::create_service(db.clone());

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

    let auth_routes = torii_axum::routes(torii.clone())
        .with_cookie_config(CookieConfig::development())
        .with_link_config(LinkConfig::new(&config.base_url))
        .build()
        .with_state(());

    let state = AppState::new(db, config, torii, instance_service);

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
