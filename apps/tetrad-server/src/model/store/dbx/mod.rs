mod error;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use super::Db;
pub use error::Error;
use sqlx::{FromRow, IntoArguments, Sqlite, Transaction, query::QueryAs, sqlite::SqliteRow};

#[derive(Clone)]
pub struct Dbx {
    db_pool: Db,
    txn_holder: Arc<tokio::sync::Mutex<Option<TxnHolder>>>,
    with_txn: bool,
}

impl Dbx {
    pub fn new(db_pool: Db, with_txn: bool) -> Result<Self, Error> {
        Ok(Dbx {
            db_pool,
            txn_holder: Arc::default(),
            with_txn,
        })
    }

    pub async fn begin_txn(&self) -> Result<(), Error> {
        if !self.with_txn {
            return Err(Error::CannotBeginTxnWithTxnFalse);
        }

        let mut txh_g = self.txn_holder.lock().await;

        if let Some(txh) = txh_g.as_mut() {
            txh.increment();
        } else {
            let transaction = self.db_pool.begin().await?;
            let _ = txh_g.insert(TxnHolder::new(transaction));
        }
        Ok(())
    }
    pub async fn fetch_one<'q, O, A>(&self, query: QueryAs<'q, Sqlite, O, A>) -> Result<O, Error>
    where
        O: for<'r> FromRow<'r, SqliteRow> + Send + Unpin,
        A: IntoArguments<Sqlite> + 'q,
    {
        let data = if self.with_txn {
            let mut txh_g = self.txn_holder.lock().await;
            if let Some(txn) = txh_g.as_deref_mut() {
                query.fetch_one(txn.as_mut()).await?
            } else {
                query.fetch_one(self.db()).await?
            }
        } else {
            query.fetch_one(self.db()).await?
        };

        Ok(data)
    }

    pub fn db(&self) -> &Db {
        &self.db_pool
    }
}

struct TxnHolder {
    txn: Transaction<'static, Sqlite>,
    counter: i32,
}

impl Deref for TxnHolder {
    type Target = Transaction<'static, Sqlite>;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}

impl DerefMut for TxnHolder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.txn
    }
}

impl TxnHolder {
    fn new(txn: Transaction<'static, Sqlite>) -> Self {
        Self { txn, counter: 1 }
    }

    fn increment(&mut self) {
        self.counter += 1;
    }

    fn decrement(&mut self) -> i32 {
        self.counter -= 1;
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Duration;

    fn test_dbx(with_txn: bool) -> Dbx {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy("sqlite::memory:")
            .expect("valid db url");
        Dbx::new(pool, with_txn).expect("create Dbx")
    }

    // The table is committed; the inserted row remains in an open transaction.
    async fn dbx_with_uncommitted_insert() -> Dbx {
        let dbx = test_dbx(true);

        sqlx::query(
            "CREATE TABLE lifecycle_items (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .execute(dbx.db())
        .await
        .expect("create table outside transaction");

        dbx.begin_txn().await.expect("begin transaction");

        let row: (i64,) = dbx
            .fetch_one(
                sqlx::query_as(
                    "INSERT INTO lifecycle_items (id, name)
                     VALUES (1, ?)
                     RETURNING id",
                )
                .bind("pending"),
            )
            .await
            .expect("insert inside transaction");

        assert_eq!(row, (1,));

        let row: (String,) = dbx
            .fetch_one(sqlx::query_as(
                "SELECT name FROM lifecycle_items WHERE id = 1",
            ))
            .await
            .expect("inserted row exists inside transaction");

        assert_eq!(row.0, "pending");

        dbx
    }

    #[tokio::test]
    async fn clone_shares_open_transaction() {
        let dbx = dbx_with_uncommitted_insert().await;
        let cloned = dbx.clone();

        let row: (String,) = cloned
            .fetch_one(sqlx::query_as(
                "SELECT name FROM lifecycle_items WHERE id = 1",
            ))
            .await
            .expect("clone reads uncommitted insert");

        assert_eq!(row.0, "pending");
    }

    #[tokio::test]
    async fn dropping_one_owner_keeps_transaction_alive() {
        let dbx = dbx_with_uncommitted_insert().await;
        let surviving_clone = dbx.clone();

        drop(dbx);

        let row: (String,) = surviving_clone
            .fetch_one(
                sqlx::query_as(
                    "UPDATE lifecycle_items
                     SET name = ?
                     WHERE id = 1
                     RETURNING name",
                )
                .bind("still active"),
            )
            .await
            .expect("surviving clone can update uncommitted row");

        assert_eq!(row.0, "still active");
    }

    #[tokio::test]
    async fn dropping_final_owner_rolls_back_transaction() {
        let dbx = dbx_with_uncommitted_insert().await;

        // Retain database access without retaining the transaction holder.
        let pool = dbx.db().clone();
        let cloned = dbx.clone();

        drop(dbx);
        drop(cloned);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lifecycle_items")
            .fetch_one(&pool)
            .await
            .expect("query after transaction rollback");

        assert_eq!(count, 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn begin_txn_rejects_disabled_transactions() {
        let dbx = test_dbx(false);
        let result = dbx.begin_txn().await;

        assert!(matches!(result, Err(Error::CannotBeginTxnWithTxnFalse)));
        assert!(dbx.txn_holder.lock().await.is_none());
    }

    #[tokio::test]
    async fn begin_txn_opens_transaction() {
        let dbx = test_dbx(true);
        dbx.begin_txn().await.expect("begin transaction");

        let txh_g = dbx.txn_holder.lock().await;
        let txn = txh_g.as_ref().expect("transaction exists");
        assert_eq!(txn.counter, 1);
    }

    #[tokio::test]
    async fn begin_txn_again_increments_counter() {
        let dbx = test_dbx(true);
        dbx.begin_txn().await.expect("begin transaction");
        dbx.begin_txn().await.expect("nested begin transaction");

        let txh_g = dbx.txn_holder.lock().await;
        let txn = txh_g.as_ref().expect("transaction exists");
        assert_eq!(txn.counter, 2);
    }

    #[tokio::test]
    async fn begin_txn_propogates_pool_error() {
        let dbx = test_dbx(true);
        dbx.db().close().await;
        let result = dbx.begin_txn().await;

        assert!(matches!(result, Err(Error::Sqlx(sqlx::Error::PoolClosed))));
        assert!(dbx.txn_holder.lock().await.is_none());
    }

    #[tokio::test]
    async fn fetch_one_uses_pool_when_transactions_disabled() {
        let dbx = test_dbx(false);

        let row: (i64,) = dbx
            .fetch_one(sqlx::query_as("SELECT 42"))
            .await
            .expect("fetch row");

        assert_eq!(row, (42,));
    }

    #[tokio::test]
    async fn fetch_one_uses_pool_when_no_transaction_is_open() {
        let dbx = test_dbx(true);

        let row: (i64,) = dbx
            .fetch_one(sqlx::query_as("SELECT 42"))
            .await
            .expect("fetch row");

        assert_eq!(row, (42,));
        assert!(dbx.txn_holder.lock().await.is_none());
    }

    #[tokio::test]
    async fn fetch_one_uses_open_transaction() {
        let dbx = test_dbx(true);
        dbx.begin_txn().await.expect("begin transaction");

        let row: (i64,) = dbx
            .fetch_one(sqlx::query_as("SELECT 42"))
            .await
            .expect("fetch through transaction");

        assert_eq!(row, (42,));
    }

    #[tokio::test]
    async fn fetch_one_propagates_row_not_found() {
        let dbx = test_dbx(false);

        let result = dbx
            .fetch_one(sqlx::query_as::<_, (i64,)>("SELECT 42 WHERE 1 = 0"))
            .await;

        assert!(matches!(result, Err(Error::Sqlx(sqlx::Error::RowNotFound))));
    }

    #[tokio::test]
    async fn fetch_one_preserves_bound_sql_looking_text() {
        let dbx = test_dbx(false);
        let input = "' OR 1=1; --";

        let row: (String,) = dbx
            .fetch_one(sqlx::query_as("SELECT ?").bind(input))
            .await
            .expect("fetch bound value");

        assert_eq!(row.0, input);
    }
}
