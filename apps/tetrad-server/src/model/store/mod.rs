use sqlx::{Pool, Sqlite, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};

pub(in crate::model) mod dbx;

pub type Db = Pool<Sqlite>;

pub async fn new_db_pool(options: SqliteConnectOptions) -> sqlx::Result<Db> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}
