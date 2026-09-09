#![allow(unused)]

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    TxnCantCommitNoOpenTxn,
    #[error("{self:?}")]
    CannotBeginTxnWithTxnFalse,
    #[error("{self:?}")]
    CannotCommitTxnWithTxnFalse,
    #[error("{self:?}")]
    NoTxn,

    #[error("{self:?}")]
    Sqlx(#[from] sqlx::Error),
}
