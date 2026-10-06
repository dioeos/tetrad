#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    InvalidBindAddress(#[from] std::net::AddrParseError),
}
