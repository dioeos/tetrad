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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_fail_to_parse_db_url_error_on_invalid_url() {
        let url = "sqlite::memory:?mode=invalid";
        let result = new_db_pool(url).await;

        assert!(matches!(result, Err(Error::FailToParseDatabaseUrl(_))));
    }

    #[tokio::test]
    async fn returns_fail_to_create_pool_when_parent_directory_is_missing() {
        let missing_dir =
            std::env::temp_dir().join(format!("tetrad-missing-{}", uuid::Uuid::now_v7()));

        assert!(!missing_dir.exists());

        let db_path = missing_dir.join("test.sqlite3");
        let url = format!("sqlite://{}", db_path.display());

        let result = new_db_pool(&url).await;

        assert!(matches!(result, Err(Error::FailToCreatePool(_))));
    }
}
