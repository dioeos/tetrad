use tetrad_server::{Config, run, use_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = use_config();
    run(config).await
}
