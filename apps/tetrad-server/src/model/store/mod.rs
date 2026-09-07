use std::{str::FromStr, time::Duration};

use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub(in crate::model) mod dbx;
pub(in crate::model) mod error;

use error::Error;
pub type Db = Pool<Sqlite>;

pub async fn new_db_pool(database_url: &str) -> Result<Db, Error> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(Error::FailToParseDatabaseUrl)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(Error::FailToCreatePool)
}
