//! `MemoryRepo` — fake in-memory che implementa [`LifecycleRepo`].
//!
//! Usato per i test dei caller senza bisogno di un database reale.
//! Restituisce il set seed noto (5 stati, 7 transizioni).

use std::sync::Arc;

use crate::domain::transition::valid_transitions;
use crate::domain::{Status, Transition};
use crate::repo::{LifecycleRepo, RepoError};

/// Fake repository in-memory per il lifecycle.
///
/// Restituisce sempre lo stesso set seed (5 stati, 7 transizioni),
/// indipendentemente da qualsiasi operazione di scrittura.
#[derive(Debug, Clone)]
pub struct MemoryRepo {
    states: Arc<Vec<Status>>,
    transitions: Arc<Vec<Transition>>,
}

impl MemoryRepo {
    /// Crea un `MemoryRepo` con stati e transizioni personalizzati.
    pub fn with_seed(states: Vec<Status>, transitions: Vec<Transition>) -> Self {
        Self {
            states: Arc::new(states),
            transitions: Arc::new(transitions),
        }
    }

    /// Crea un `MemoryRepo` con il set seed completo (5 stati, 7 transizioni).
    pub fn default_seed() -> Self {
        Self {
            states: Arc::new(vec![
                Status::Active,
                Status::OnHold,
                Status::Abandoned,
                Status::Done,
                Status::Superseded,
            ]),
            transitions: Arc::new(valid_transitions()),
        }
    }
}

impl Default for MemoryRepo {
    fn default() -> Self {
        Self::default_seed()
    }
}

impl LifecycleRepo for MemoryRepo {
    fn states(&self) -> Result<Vec<Status>, RepoError> {
        Ok(self.states.as_ref().clone())
    }

    fn allowed_transitions(&self) -> Result<Vec<Transition>, RepoError> {
        Ok(self.transitions.as_ref().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_returns_seed_states() {
        let repo = MemoryRepo::default();
        let states = repo.states().expect("states should succeed");
        assert_eq!(states.len(), 5);
        assert!(states.contains(&Status::Active));
        assert!(states.contains(&Status::Done));
    }

    #[test]
    fn memory_returns_seed_transitions() {
        let repo = MemoryRepo::default();
        let transitions = repo
            .allowed_transitions()
            .expect("transitions should succeed");
        assert_eq!(transitions.len(), 7);
    }

    #[test]
    fn memory_with_custom_seed() {
        let repo = MemoryRepo::with_seed(vec![Status::Active, Status::Done], vec![]);
        let states = repo.states().expect("states should succeed");
        assert_eq!(states.len(), 2);
        let transitions = repo
            .allowed_transitions()
            .expect("transitions should succeed");
        assert!(transitions.is_empty());
    }
}
