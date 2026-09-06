mod store;
mod error;

use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::SqliteConnectOptions;
use store::dbx::Dbx;
use store::new_db_pool;
use error::Error;

use crate::config::use_config;

#[derive(Clone)]
pub struct ModelManager {
    dbx: Dbx
}

impl ModelManager {
    pub async fn new() -> Result<Self, Error> {
        let options = SqliteConnectOptions::from_str(&use_config().database_url)
            .map_err(Error::CantConnectToSqlite)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        
        let db_pool = new_db_pool(options)
            .await
            .map_err(|err| Error::CantCreateModelManagerProvider(err.to_string()))?;

        let dbx = Dbx::new(db_pool, false)?;
        Ok(ModelManager { dbx })
    }

    // pub async fn new_with_txn(&self) -> Result<Self, Error> {
    //     let dbx = Dbx::new(self.
    // }
}
