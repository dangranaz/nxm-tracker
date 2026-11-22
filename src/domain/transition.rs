//! `Transition` and `Trigger` — valid state transitions for the lifecycle.
//!
//! Defines the trigger enum (`set_status` | `reopen`) and the `Transition`
//! value type (from → to via trigger), with parsing and validation logic.
//! Matches the `lifecycle_transition` table seeded in migration v5-v6.
//! Pure domain: no I/O, no database.

use std::convert::TryInto;
use std::fmt;

use thiserror::Error;

use super::status::Status;

/// Trigger causing a lifecycle state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    /// Explicit status change (`set_status`).
    SetStatus,
    /// Reopen a suspended/archived session (`reopen`).
    Reopen,
}

impl Trigger {
    /// Returns the lowercase string used for DB storage.
    pub fn as_str(self) -> &'static str {
        match self {
            Trigger::SetStatus => "set_status",
            Trigger::Reopen => "reopen",
        }
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for Trigger {
    type Error = ParseTriggerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "set_status" => Ok(Trigger::SetStatus),
            "reopen" => Ok(Trigger::Reopen),
            other => Err(ParseTriggerError(other.to_string())),
        }
    }
}

/// A permitted lifecycle state transition: `from_state` → `to_state` via
/// `trigger`. Matches a row in the `lifecycle_transition` table.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transition {
    pub from: Status,
    pub to: Status,
    pub trigger: Trigger,
    pub description: String,
}

impl Transition {
    /// Returns `true` if this transition targets a terminal state.
    pub fn targets_terminal(self) -> bool {
        self.to.is_terminal()
    }
}

/// Error parsing a trigger string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown trigger: {0}")]
pub struct ParseTriggerError(pub String);

/// Returns the known set of valid transitions, seeded in migration v6.
pub fn valid_transitions() -> Vec<Transition> {
    vec![
        Transition {
            from: Status::Active,
            to: Status::OnHold,
            trigger: Trigger::SetStatus,
            description: "Park for a collision between sessions on the same project.".to_string(),
        },
        Transition {
            from: Status::Active,
            to: Status::Abandoned,
            trigger: Trigger::SetStatus,
            description: "Abandon: dormant in place, not archived.".to_string(),
        },
        Transition {
            from: Status::Active,
            to: Status::Done,
            trigger: Trigger::SetStatus,
            description: "Finished with value: archived, not reopenable.".to_string(),
        },
        Transition {
            from: Status::Active,
            to: Status::Superseded,
            trigger: Trigger::SetStatus,
            description: "Replaced by a successor: archived, reopenable.".to_string(),
        },
        Transition {
            from: Status::OnHold,
            to: Status::Active,
            trigger: Trigger::Reopen,
            description: "Resume from the parked state.".to_string(),
        },
        Transition {
            from: Status::Abandoned,
            to: Status::Active,
            trigger: Trigger::Reopen,
            description: "Resume in place.".to_string(),
        },
        Transition {
            from: Status::Superseded,
            to: Status::Active,
            trigger: Trigger::Reopen,
            description: "Resume with a physical return of files from archived/.".to_string(),
        },
    ]
}

/// Checks whether a transition `from` → `to` via `trigger` is permitted.
pub fn is_transition_allowed(from: Status, to: Status, trigger: Trigger) -> bool {
    valid_transitions()
        .iter()
        .any(|t| t.from == from && t.to == to && t.trigger == trigger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions_match_seed_count() {
        assert_eq!(valid_transitions().len(), 7);
    }

    #[test]
    fn is_transition_allowed_known() {
        assert!(is_transition_allowed(
            Status::Active,
            Status::Done,
            Trigger::SetStatus
        ));
        assert!(is_transition_allowed(
            Status::OnHold,
            Status::Active,
            Trigger::Reopen
        ));
        assert!(is_transition_allowed(
            Status::Superseded,
            Status::Active,
            Trigger::Reopen
        ));
    }

    #[test]
    fn is_transition_allowed_unknown() {
        assert!(!is_transition_allowed(
            Status::Done,
            Status::Active,
            Trigger::Reopen
        ));
        assert!(!is_transition_allowed(
            Status::Active,
            Status::OnHold,
            Trigger::Reopen
        ));
    }

    #[test]
    fn trigger_try_from_string_roundtrip() {
        for t in [Trigger::SetStatus, Trigger::Reopen] {
            let parsed: Trigger = t.as_str().try_into().unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn trigger_try_from_invalid_errors() {
        let err: ParseTriggerError = TryInto::<Trigger>::try_into("invalid").unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }
}
