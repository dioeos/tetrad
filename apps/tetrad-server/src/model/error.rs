use super::store;

//@NOTE: DBX is a nested module inside store. However, there are separate
//       error modules for each. This is because the store's only isolated interaction
//       is when a pool connection is created via `new_db_pool()`, which is only
//       called in `ModelManager::new()`. From that point after, the model manager has
//       direct access to DBX through this store connection. Model controllers directly
//       interact with the Model Manager's DBX via `mm.dbx()`, which is why the errors
//       are separated, as the model controllers must wrap these direct DBX errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // model manager + store
    #[error("{self:?}")]
    CantCreateModelManagerProviderDbPool(#[source] store::error::Error),
    #[error("{self:?}")]
    CantMigrateManagerProviderDb(#[source] sqlx::migrate::MigrateError),

    // instance bmc
    #[error("failed to insert the singleton instance")]
    FailedToInsertInstance(#[source] store::dbx::Error),
    #[error("failed to retrieve the existing singleton instance")]
    FailedToGetExistingInstance(#[source] store::dbx::Error),
    #[error("{self:?}")]
    InvalidInstanceTimestamp(#[from] time::error::ComponentRange),
    #[error("{self:?}")]
    InvalidInstanceUuid(#[from] uuid::Error),

    // internal module errors
    #[error("{self:?}")]
    Dbx(#[from] store::dbx::Error),

    //@NOTE: The seaquery and sqlx errors at the model level can come from either base crud functions
    //       or within the bmcs directly. For example, the `instance` module can propogate both
    //       since it makes direct use of base crud functions as well as implementing its own SQL
    //       functionality
    //
    // externals
    #[error("{self:?}")]
    SeaQuery(#[from] sea_query::error::Error),

    #[error("{self:?}")]
    Sqlx(#[from] sqlx::Error),
}
