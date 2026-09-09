#[derive(Debug, thiserror::Error)]
pub enum Error {
    // config
    #[error("{self:?}")]
    InvalidBindAddress(#[from] std::net::AddrParseError),

    // internal modules
}

