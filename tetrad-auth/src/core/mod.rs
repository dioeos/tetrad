pub mod error;
pub mod login;
pub mod identity;
pub mod session;

use std::fmt::Debug;
use error::PhaseError;

pub trait Phase: Copy + Debug + PartialEq + Eq {
    fn allowed_next(self) -> &'static [Self];
    fn is_terminal(self) -> bool;
    fn transition_to(&mut self, next_phase: Self) -> Result<(), PhaseError<Self>> {
        if self.is_terminal() {
            return Err(PhaseError::CannotTransitionFromTerminalState { current: *self });
        }

        let allowed = self.allowed_next();
        if !allowed.contains(&next_phase) {
            return Err(PhaseError::InvalidTransition {
                current: *self,
                attempted: next_phase,
                allowed
            })
        }
        *self = next_phase;
        Ok(())
    }
}
