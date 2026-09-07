mod base;
mod error;
mod store;

// entity models + bmcs
mod instance;

use error::Error;
use store::dbx::Dbx;
use store::new_db_pool;

#[derive(Clone)]
pub struct ModelManager {
    dbx: Dbx,
}

impl ModelManager {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let db_pool = new_db_pool(database_url)
            .await
            .map_err(Error::CantCreateModelManagerProvider)?;

        let dbx = Dbx::new(db_pool, false)?;
        Ok(ModelManager { dbx })
    }

    pub(in crate::model) fn dbx(&self) -> &Dbx {
        &self.dbx
    }

    // pub async fn new_with_txn(&self) -> Result<Self, Error> {
    //     let dbx = Dbx::new(self.
    // }
}
