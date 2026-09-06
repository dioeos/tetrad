use super::store::dbx;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // model manager
    #[error("{self:?}")]
    CantCreateModelManagerProvider(String),

    #[error("{self:?}")]
    CantConnectToSqlite(#[source] sqlx::Error),

    // internal module errors
    #[error("{self:?}")]
    Dbx(#[from] dbx::Error),
}


