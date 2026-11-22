//! `SessionView` — vista di lettura di una sessione.

use crate::domain::Status;

/// Vista di lettura di una sessione di lavoro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionView {
    pub id: String,
    pub name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: Status,
    pub design_phase_complete: bool,
    pub consumed_at: Option<String>,
    pub consumed_by: Option<String>,
}

impl SessionView {
    /// Costruisce una `SessionView` da una riga del database.
    pub fn from_row(
        name: String,
        started_at: String,
        ended_at: Option<String>,
        status: Status,
        design_phase_complete: bool,
        consumed_at: Option<String>,
        consumed_by: Option<String>,
    ) -> Self {
        Self {
            id: String::new(), // sarà impostato dal chiamante
            name,
            started_at,
            ended_at,
            status,
            design_phase_complete,
            consumed_at,
            consumed_by,
        }
    }
}
