use std::{str::FromStr, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use torii_storage_sqlite::SqliteStorage;

async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub(super) async fn initialize(database_url: &str) -> anyhow::Result<(SqlitePool, SqliteStorage)> {
    let db: SqlitePool = connect(database_url).await?;

    let storage = SqliteStorage::new(db.clone());
    storage.migrate().await?;
    migrate(&db).await?;

    Ok((db, storage))
}
