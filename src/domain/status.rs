//! `Status` — gli stati canonici del ciclo di vita di una sessione.
//!
//! Definisce l'enum `Status` (5 stati: `Active`, `OnHold`, `Abandoned`,
//! `Done`, `Superseded`) con i predicati `is_visible_default`,
//! `is_archivable`, `is_reopenable` e la serializzazione da/per stringa.
//! Nessun I/O, nessun database: solo tipi e funzioni pure.

use std::convert::TryInto;
use std::fmt;

/// Lifecycle status of a work session.
///
/// Canonical set (5 states, matching `lifecycle_state` table):
/// `Active` → `OnHold` → `Abandoned`/`Done`/`Superseded`, with reopen paths
/// from `OnHold`, `Abandoned`, `Superseded` back to `Active`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// In progress / resumable — the live work queue.
    Active,
    /// Parked for a collision between sessions on the same project.
    OnHold,
    /// Dormant in place; not archived; hidden from the default listing.
    Abandoned,
    /// Finished with value; archived immediately; not reopenable.
    Done,
    /// Replaced by a successor; archived but reopenable.
    Superseded,
}

impl Status {
    /// Returns the lowercase string used for DB storage and MCP payloads.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::OnHold => "onhold",
            Status::Abandoned => "abandoned",
            Status::Done => "done",
            Status::Superseded => "superseded",
        }
    }

    /// Is this session visible in the default `list_sessions` view?
    pub fn is_visible_default(self) -> bool {
        matches!(self, Status::Active | Status::OnHold)
    }

    /// Can this session be archived (moved to `archived/`)?
    pub fn is_archivable(self) -> bool {
        matches!(self, Status::Done | Status::Superseded)
    }

    /// Can this session be reopened back to `Active`?
    pub fn is_reopenable(self) -> bool {
        matches!(
            self,
            Status::Active | Status::OnHold | Status::Abandoned | Status::Superseded
        )
    }

    /// Is this a terminal state (no outgoing transitions)?
    pub fn is_terminal(self) -> bool {
        matches!(self, Status::Done)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for Status {
    type Error = ParseStatusError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Status::Active),
            "onhold" => Ok(Status::OnHold),
            "abandoned" => Ok(Status::Abandoned),
            "done" => Ok(Status::Done),
            "superseded" => Ok(Status::Superseded),
            other => Err(ParseStatusError(other.to_string())),
        }
    }
}

/// Error parsing a status string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown status: {0}")]
pub struct ParseStatusError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_as_str_matches_db_values() {
        assert_eq!(Status::Active.as_str(), "active");
        assert_eq!(Status::OnHold.as_str(), "onhold");
        assert_eq!(Status::Abandoned.as_str(), "abandoned");
        assert_eq!(Status::Done.as_str(), "done");
        assert_eq!(Status::Superseded.as_str(), "superseded");
    }

    #[test]
    fn status_predicates_match_seed_data() {
        assert!(Status::Active.is_visible_default());
        assert!(!Status::Active.is_archivable());
        assert!(Status::Active.is_reopenable());

        assert!(Status::OnHold.is_visible_default());
        assert!(!Status::OnHold.is_archivable());
        assert!(Status::OnHold.is_reopenable());

        assert!(!Status::Abandoned.is_visible_default());
        assert!(!Status::Abandoned.is_archivable());
        assert!(Status::Abandoned.is_reopenable());

        assert!(!Status::Done.is_visible_default());
        assert!(Status::Done.is_archivable());
        assert!(!Status::Done.is_reopenable());
        assert!(Status::Done.is_terminal());

        assert!(!Status::Superseded.is_visible_default());
        assert!(Status::Superseded.is_archivable());
        assert!(Status::Superseded.is_reopenable());
    }

    #[test]
    fn status_try_from_string_roundtrip() {
        for s in [
            Status::Active,
            Status::OnHold,
            Status::Abandoned,
            Status::Done,
            Status::Superseded,
        ] {
            let parsed: Status = s.as_str().try_into().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn status_try_from_invalid_errors() {
        let err: ParseStatusError = TryInto::<Status>::try_into("invalid").unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }
}
