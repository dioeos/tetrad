use tetrad_server::{Config, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "tetrad_server=debug,tower_http=debug".into())
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();
    // let database_url = std::env::var("DATABASE_URL")
    //     .unwrap_or_else(|_| "sqlite://data/tetrad.sqlite3".to_owned());
    //
    // let connect_options = SqliteConnectOptions::from_str(&database_url)?
    //     .create_if_missing(true)
    //     .foreign_keys(true)
    //     .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
    //     .busy_timeout(Duration::from_secs(5));
    //
    // let db_pool = SqlitePoolOptions::new()
    //     .max_connections(3)
    //     .connect_with(connect_options)
    //     .await?;
    //
    // sqlx::migrate!().run(&db_pool).await?;
    //
    // let state = AppState { db: db_pool };
    //
    // let app = Router::new()
    //     .route("/", get(|| async { "Hello, World!" }))
    //     .with_state(state)
    //     .layer(TraceLayer::new_for_http());
    //
    // let addr = "0.0.0.0:3000";
    //
    // let listener = tokio::net::TcpListener::bind(addr)
    //     .await
    //     .unwrap();
    //
    // info!("server listening on http://{}", addr);
    //
    // axum::serve(listener, app)
    //     .await
    //     .unwrap();
    //
    let config = Config::from_environment()?;
    run(config).await
}
