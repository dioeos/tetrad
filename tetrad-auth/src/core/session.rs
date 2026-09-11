use super::{identity::AccountIdentity, login::OperationId, error::{Error, PhaseError}, Phase};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SessionPhase {
    Active,
    LoggingOut,
    Closed,
}

impl Phase for SessionPhase {
    fn allowed_next(self) -> &'static [Self] {
        use SessionPhase::*;
        match self {
            Active => &[LoggingOut],
            LoggingOut => &[Active, Closed],
            Closed => &[]
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Closed)
    }
}

//@NOTE: A session's `id` comes from successful login/restoration
//       request.
pub struct Session {
    id: OperationId,
    account: AccountIdentity,
    phase: SessionPhase,
}

impl Session {
    pub fn new(id: OperationId, account: AccountIdentity) -> Self {
        Self {
            id,
            account,
            phase: SessionPhase::Active,
        }
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn account(&self) -> &AccountIdentity {
        &self.account
    }

    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn is_authenticated(&self) -> bool {
        //SessionPhase::LoggingOut is a temporary processing state so user is still
        //considered to be authenticated during this state
        matches!(self.phase, SessionPhase::Active | SessionPhase::LoggingOut)
    }

    pub fn begin_logout(&mut self) -> Result<(), Error> {
        if self.phase != SessionPhase::Active {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: SessionPhase::Active
            })?;
        }

        self.phase.transition_to(SessionPhase::LoggingOut)?;

        Ok(())
    }

    pub fn credential_deletion_failed(&mut self) -> Result<(), Error> {
        if self.phase != SessionPhase::LoggingOut {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: SessionPhase::LoggingOut
            })?;
        }

        self.phase.transition_to(SessionPhase::Active)?;

        Ok(())
    }

    pub fn credential_deletion_completed(&mut self) -> Result<(), Error> {
        if self.phase != SessionPhase::LoggingOut {
            return Err(PhaseError::InvalidPhase {
                current: self.phase,
                expected: SessionPhase::LoggingOut
            })?;
        }

        self.phase.transition_to(SessionPhase::Closed)?;

        Ok(())
    }
}
