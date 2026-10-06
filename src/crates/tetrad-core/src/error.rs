use super::model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    //config
    #[error("{self:?}")]
    InvalidBindAddress(#[from] std::net::AddrParseError),

    //model internals
    #[error(transparent)]
    ModelManager(#[from] model::error::Error),
}
