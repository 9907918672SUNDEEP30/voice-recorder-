//! # Explicit State Machine
//!
//! The engine is a *blind* runner: it never inspects what a step does, but it
//! always knows exactly where it is. That guarantee lives here.
//!
//! Transitions:
//! * Start  -> Running
//! * Running-> Success | Retry | Failed
//! * Retry  -> Running | Failed
//! * Terminal states: Success, Failed (closed)

/// Every state the orchestrator may occupy or observe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Job accepted, nothing executed yet.
    Start,
    /// A step attempt is in flight.
    Running,
    /// Terminal happy path.
    Success,
    /// A step attempt failed and retry is allowed.
    Retry,
    /// Terminal unhappy path.
    Failed,
}

impl State {
    /// Single source of truth for legal transitions.
    pub fn can_transition_to(self, next: State) -> bool {
        match self {
            State::Start => matches!(next, State::Running),
            State::Running => matches!(next, State::Success | State::Retry | State::Failed),
            State::Retry => matches!(next, State::Running | State::Failed),
            State::Success | State::Failed => false,
        }
    }

    /// A terminal state ends the run: nothing may execute afterwards.
    pub fn is_terminal(self) -> bool {
        matches!(self, State::Success | State::Failed)
    }

    /// Lowercase, stable name used by the telemetry line format.
    pub fn label(self) -> &'static str {
        match self {
            State::Start => "start",
            State::Running => "running",
            State::Success => "success",
            State::Retry => "retry",
            State::Failed => "failed",
        }
    }
}

/// Error raised when a caller tries to drive the machine into an illegal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalTransition {
    pub from: State,
    pub to: State,
}

impl std::fmt::Display for IllegalTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "illegal state transition: {} -> {}",
            self.from.label(),
            self.to.label()
        )
    }
}

impl std::error::Error for IllegalTransition {}

/// A checked transition helper so no code path can bypass the rules above.
pub fn transition(from: State, to: State) -> Result<State, IllegalTransition> {
    if from.can_transition_to(to) {
        Ok(to)
    } else {
        Err(IllegalTransition { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_every_documented_transition() {
        assert!(State::Start.can_transition_to(State::Running));
        assert!(State::Running.can_transition_to(State::Success));
        assert!(State::Running.can_transition_to(State::Retry));
        assert!(State::Running.can_transition_to(State::Failed));
        assert!(State::Retry.can_transition_to(State::Running));
        assert!(State::Retry.can_transition_to(State::Failed));
    }

    #[test]
    fn rejects_illegal_transitions() {
        assert!(!State::Start.can_transition_to(State::Success));
        assert!(!State::Start.can_transition_to(State::Retry));
        assert!(!State::Start.can_transition_to(State::Failed));
        assert!(!State::Retry.can_transition_to(State::Success));
        assert!(!State::Success.can_transition_to(State::Running));
        assert!(!State::Failed.can_transition_to(State::Running));
    }

    #[test]
    fn terminal_states_are_closed() {
        assert!(State::Success.is_terminal());
        assert!(State::Failed.is_terminal());
        assert!(!State::Start.is_terminal());
        assert!(!State::Running.is_terminal());
        assert!(!State::Retry.is_terminal());
    }

    #[test]
    fn checked_transition_reports_the_violation() {
        assert_eq!(transition(State::Start, State::Running), Ok(State::Running));
        let err = transition(State::Start, State::Success).unwrap_err();
        assert_eq!(
            err,
            IllegalTransition {
                from: State::Start,
                to: State::Success
            }
        );
        assert_eq!(err.to_string(), "illegal state transition: start -> success");
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(State::Start.label(), "start");
        assert_eq!(State::Running.label(), "running");
        assert_eq!(State::Success.label(), "success");
        assert_eq!(State::Retry.label(), "retry");
        assert_eq!(State::Failed.label(), "failed");
    }
}
