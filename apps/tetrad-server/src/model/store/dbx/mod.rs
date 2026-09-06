mod error;

use std::sync::Arc;

use super::Db;
pub use error::Error;
use sqlx::{Sqlite, Transaction};

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
}

struct TxnHolder {
    txn: Transaction<'static, Sqlite>,
    counter: i32,
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
