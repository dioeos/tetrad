use tetrad_server::{Config, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_environment()?;
    run(config).await
}
