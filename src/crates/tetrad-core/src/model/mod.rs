pub(super) mod error;
mod store;

use std::path::Path;

use error::Error;
use sqlx::migrate::Migrator;

use crate::model::store::dbx::Dbx;

pub struct ModelManager {
    dbx: Dbx,
}

impl ModelManager {
    pub async fn new(database_url: &str, migrations_path: &str) -> Result<Self, Error> {
        let db_pool = store::new_db_pool(database_url)
            .await
            .map_err(Error::CreateModelManagerProviderDbPool)?;

        let migrator = Migrator::new(Path::new(migrations_path))
            .await
            .map_err(Error::LoadMigrations)?;

        migrator
            .run(&db_pool)
            .await
            .map_err(Error::MigrateManagerProviderDb)?;

        let dbx = Dbx::new(db_pool, false);
        Ok(ModelManager { dbx })
    }
}
