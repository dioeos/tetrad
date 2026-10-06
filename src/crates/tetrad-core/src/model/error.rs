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
    #[error("{self:?}")]
    CreateModelManagerProviderDbPool(#[source] store::error::Error), 

    #[error("{self:?}")]
    LoadMigrations(#[source] sqlx::migrate::MigrateError),

    #[error("{self:?}")]
    MigrateManagerProviderDb(#[source] sqlx::migrate::MigrateError),

    //internal module errors
    #[error("{self:?}")]
    Dbx(#[from] store::dbx::error::Error)
}
