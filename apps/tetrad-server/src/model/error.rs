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
    CantCreateModelManagerProvider(#[source] store::error::Error),

    // instace bmc
    #[error("{self:?}")]
    InvalidInstanceTimestamp(#[from] time::error::ComponentRange),
    #[error("{self:?}")]
    InvalidInstanceUuid(#[from] uuid::Error),

    // internal module errors
    #[error("{self:?}")]
    Dbx(#[from] store::dbx::Error),

    // externals
    #[error("{self:?}")]
    SeaQuery(#[from] sea_query::error::Error),

    #[error("{self:?}")]
    Sqlx(#[from] sqlx::Error)
}
