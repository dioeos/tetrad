use std::time::Instant;

use super::{Phase, login::LoginPhase, session::SessionPhase};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("{self:?}")]
    LoginPhaseError(#[from] PhaseError<LoginPhase>),
    #[error("{self:?}")]
    SessionPhaseError(#[from] PhaseError<SessionPhase>),

    #[error("{self:?}")]
    LoginExpired {
        expires_at: Instant,
        checked_at: Instant,
    },

    #[error("Missing callback state")]
    MissingCallbackState,

    #[error("Callback does not match login attempt")]
    CallbackStateMismatch,

    #[error("Callback state must not be empty")]
    EmptyCallbackState,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PhaseError<P: Phase + 'static> {
    #[error("{self:?}")]
    InvalidTransition {
        current: P,
        attempted: P,
        allowed: &'static [P]
    },

    #[error("{self:?}")]
    InvalidPhase {
        current: P,
        expected: P
    },

    #[error("{self:?}")]
    CannotTransitionFromTerminalState {
        current: P
    },
}
