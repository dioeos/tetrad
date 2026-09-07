use super::store;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // model manager + store
    #[error("{self:?}")]
    CantCreateModelManagerProvider(#[source] store::error::Error),

    // internal module errors
    #[error("{self:?}")]
    Dbx(#[from] store::dbx::Error),

    // instace bmc
    #[error("{self:?}")]
    InvalidInstanceTimestamp(#[from] time::error::ComponentRange),

    // externals
    #[error("{self:?}")]
    Sqlx(#[from] sqlx::Error)
}
