#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    FailToParseDatabaseUrl(#[source] sqlx::Error),

    #[error("{self:?}")]
    FailToCreatePool(#[source] sqlx::Error),
}
