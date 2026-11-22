//! `Transition` and `Trigger` — valid state transitions for the lifecycle.
//!
//! Defines the trigger enum (`set_status` | `reopen`) and the `Transition`
//! value type (from → to via trigger), with parsing and validation logic.
//! Matches the `lifecycle_transition` table seeded in migration v5-v6.
//! Pure domain: no I/O, no database.

use crate::session::SessionStatus;
use crate::error::TrackerError;
use std::fmt;

use thiserror::Error;

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
/// `trigger`. Matches a row in the `lifecycle_transition` table seeded in migration v5-v6.
//! Pure domain: no I/O, no database.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transition {
    pub from: SessionStatus,
    pub to: SessionStatus,
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

/// The known set of valid transitions, seeded in migration v6.
pub fn valid_transitions() -> Vec<Transition> {
    vec![
        Transition {
            from: SessionStatus::Active,
            to: SessionStatus::OnHold,
            trigger: Trigger::SetStatus,
            description: "Park for a collision between sessions on the same project.",
        },
        Transition {
            from: SessionStatus::Active,
            to: SessionStatus::Abandoned,
            trigger: Trigger::SetStatus,
            description: "Abandon: dormant in place, not archived.",
        },
        Transition {
            from: SessionStatus::Active,
            to: SessionStatus::Done,
            trigger: Trigger::SetStatus,
            description: "Finished with value: archived, not reopenable.",
        },
        Transition {
            from: SessionStatus::Active,
            to: SessionStatus::Superseded,
            trigger: Trigger::SetStatus,
            description: "Replaced by a successor: archived, reopenable.",
        },
        Transition {
            from: SessionStatus::OnHold,
            to: SessionStatus::Active,
            trigger: Trigger::Reopen,
            description: "Resume from the parked state.",
        },
        Transition {
            from: SessionStatus::Abandoned,
            to: SessionStatus::Active,
            trigger: Trigger::Reopen,
            description: "Resume in place.",
        },
        Transition {
            from: SessionStatus::Superseded,
            to: SessionStatus::Active,
            trigger: Trigger::Reopen,
            description: "Resume with a physical return of files from archived/.",
        },
    ]
}

/// Checks whether a transition `from` → `to` via `trigger` is permitted.
pub fn is_transition_allowed(from: SessionStatus, to: SessionStatus, trigger: Trigger) -> bool {
    valid_transitions()
        .iter()
        .any(|t| t.from == from && t.to == to && t.trigger == trigger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions_has_seven_entries() {
        assert_eq!(valid_transitions().len(), 7);
    }
}