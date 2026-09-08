use tetrad_server::{run, use_config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = use_config();
    run(config).await
}
