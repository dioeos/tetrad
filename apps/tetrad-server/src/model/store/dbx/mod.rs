mod error;

use std::{ops::{Deref, DerefMut}, sync::Arc};

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
