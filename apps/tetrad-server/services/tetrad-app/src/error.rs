#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    FailedToInitModelManager(#[from] tetrad_core::error::Error),

    #[error("{self:?}")]
    Model(#[from] tetrad_core::model::Error),
}
