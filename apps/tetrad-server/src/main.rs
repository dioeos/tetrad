use tetrad_server::{Config, use_config, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = use_config();
    run(config).await
}
