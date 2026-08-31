mod config;
mod database;
mod instance;
mod state;

pub use config::Config;

use sqlx::SqlitePool;

use crate::instance::{InstanceService, Instance};

pub async fn run(config: Config) -> anyhow::Result<()> {
    let db: SqlitePool = database::connect(&config.database_url).await?;
    database::migrate(&db).await?;

    let instance_service: InstanceService = instance::create_service(db);

    let current_instance: Instance = instance_service
        .ensure_exists(&config.instance_name)
        .await?;

    tracing::info!(
        instance_id = %current_instance.id,
        instance_name = %current_instance.name,
        completed_at = current_instance.setup_completed_at_ms,
        "instance initialized"
    );

    Ok(())
}
