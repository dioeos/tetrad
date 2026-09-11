use std::time::Instant;

use secrecy::{ExposeSecret, SecretString};
use subtle::ConstantTimeEq;

use super::{
    Phase,
    error::{Error, PhaseError},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginPhase {
    Preparing,
    AwaitingCallback,
    Exchanging,
    Persisting,
    Completed,
    Cancelled,
    Failed,
}

impl Phase for LoginPhase {
    //@NOTE: `Completed`, `Cancelled`, and `Failed` are terminal states and therefore
    //       do not transition into any other phase. When these occur, a user can still
    //       attempt to log in again, but it creates a new operation ID, representing
    //       a completely new attempt in the `Preparing` phase.
    fn allowed_next(self) -> &'static [Self] {
        use LoginPhase::*;
        match self {
            Preparing => &[AwaitingCallback, Cancelled, Failed],
            AwaitingCallback => &[Exchanging, Cancelled, Failed],
            Exchanging => &[Persisting, Failed],
            Persisting => &[Completed, Failed],
            Completed | Cancelled | Failed => &[],
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}

impl LoginPhase {
    pub fn can_cancel(self) -> bool {
        self.allowed_next().contains(&Self::Cancelled)
    }
}

pub struct CallbackState(SecretString);

impl CallbackState {
    pub fn new(value: String) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::EmptyCallbackState);
        }

        Ok(Self(SecretString::from(value)))
    }

    fn matches(&self, other: &Self) -> bool {
        let expected = self.0.expose_secret().as_bytes();
        let received = other.0.expose_secret().as_bytes();

        bool::from(expected.ct_eq(received))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationId(pub u64);

pub struct LoginAttempt {
    id: OperationId,
    phase: LoginPhase,
    expires_at: Instant,
    expected_state: Option<CallbackState>,
}

impl LoginAttempt {
    pub fn new(id: OperationId, expires_at: Instant) -> Self {
        Self {
            id,
            phase: LoginPhase::Preparing,
            expires_at,
            expected_state: None,
        }
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn phase(&self) -> LoginPhase {
        self.phase
    }

    pub fn expires_at(&self) -> Instant {
        self.expires_at
    }

    pub fn cancel(&mut self) -> Result<(), Error> {
        if self.phase.is_terminal() {
            return Err(PhaseError::CannotTransitionFromTerminalState {
                current: self.phase,
            })?;
        }

        if !self.phase.can_cancel() {
            return Err(PhaseError::InvalidTransition {
                current: self.phase,
                attempted: LoginPhase::Cancelled,
                allowed: self.phase.allowed_next(),
            })?;
        }

        self.phase.transition_to(LoginPhase::Cancelled)?;
        self.expected_state = None;

        Ok(())
    }

    pub fn fail(&mut self) -> Result<(), Error> {
        if self.phase.is_terminal() {
            return Err(PhaseError::CannotTransitionFromTerminalState {
                current: self.phase,
            })?;
        }

        self.phase.transition_to(LoginPhase::Failed)?;
        self.expected_state = None;

        Ok(())
    }

    pub fn authorization_prepared(
        &mut self,
        expected_state: CallbackState,
        now: Instant,
    ) -> Result<(), Error> {
        if self.phase != LoginPhase::Preparing {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: LoginPhase::Preparing,
            })?;
        }

        if now >= self.expires_at {
            return Err(Error::LoginExpired {
                expires_at: self.expires_at,
                checked_at: now,
            });
        }

        self.expected_state = Some(expected_state);
        self.phase.transition_to(LoginPhase::AwaitingCallback)?;

        Ok(())
    }

    pub fn accept_success_callback(
        &mut self,
        received_state: &CallbackState,
        now: Instant,
    ) -> Result<(), Error> {
        self.validate_callback(received_state, now)?;

        self.phase.transition_to(LoginPhase::Exchanging)?;
        self.expected_state = None;

        Ok(())
    }

    pub fn tokens_validated(&mut self) -> Result<(), Error> {
        if self.phase != LoginPhase::Exchanging {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: LoginPhase::Exchanging,
            })?;
        }

        self.phase.transition_to(LoginPhase::Persisting)?;
        Ok(())
    }

    fn validate_callback(&self, received_state: &CallbackState, now: Instant) -> Result<(), Error> {
        if self.phase != LoginPhase::AwaitingCallback {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: LoginPhase::AwaitingCallback,
            })?;
        }

        if now >= self.expires_at {
            return Err(Error::LoginExpired {
                expires_at: self.expires_at,
                checked_at: now,
            });
        }

        let expected_callback = self
            .expected_state
            .as_ref()
            .ok_or(Error::MissingCallbackState)?;

        if !expected_callback.matches(received_state) {
            return Err(Error::CallbackStateMismatch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn state(value: &str) -> CallbackState {
        CallbackState::new(value.to_owned()).unwrap()
    }

    fn waiting_attempt(now: Instant) -> LoginAttempt {
        let mut attempt = LoginAttempt::new(OperationId(1), now + Duration::from_secs(300));

        attempt
            .authorization_prepared(state("expected-state"), now)
            .unwrap();
        attempt
    }

    #[test]
    fn wrong_state_preserves_the_attempt() {
        let now = Instant::now();
        let mut attempt = waiting_attempt(now);

        let result = attempt.accept_success_callback(&state("wrong-state"), now);

        assert!(matches!(result, Err(Error::CallbackStateMismatch)));
        assert_eq!(attempt.phase(), LoginPhase::AwaitingCallback);
    }

    #[test]
    fn accepting_callback_disables_cancellation() {
        let now = Instant::now();
        let mut attempt = waiting_attempt(now);

        attempt
            .accept_success_callback(&state("expected-state"), now)
            .unwrap();
        assert!(matches!(
            attempt.cancel(),
            Err(Error::LoginPhaseError(PhaseError::InvalidTransition { .. }))
        ));
    }
}
