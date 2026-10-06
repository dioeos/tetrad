#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    ParseDatabaseUrl(#[source] sqlx::Error),

    #[error("{self:?}")]
    CreatePool(#[source] sqlx::Error),
}
