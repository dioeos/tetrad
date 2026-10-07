mod base;
pub mod error;
pub mod instance;
pub mod ports;
pub mod services;
pub mod vendor;
mod store;

use error::Error;
use sqlx::migrate::Migrator;

use crate::model::store::dbx::Dbx;

static MIGRATOR: Migrator = sqlx::migrate!("../../server/migrations");

#[derive(Clone)]
pub struct ModelManager {
    dbx: Dbx,
}

impl ModelManager {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let db_pool = store::new_db_pool(database_url)
            .await
            .map_err(Error::CreateModelManagerProviderDbPool)?;

        MIGRATOR
            .run(&db_pool)
            .await
            .map_err(Error::MigrateManagerProviderDb)?;

        let dbx = Dbx::new(db_pool, false);
        Ok(ModelManager { dbx })
    }

    pub(in crate::model) fn dbx(&self) -> &Dbx {
        &self.dbx
    }
}
