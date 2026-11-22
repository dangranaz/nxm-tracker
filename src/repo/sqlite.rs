//! `SqliteRepo` — implementazione SQLite di [`LifecycleRepo`].
//!
//! Legge dagli tabelle `lifecycle_state` e `lifecycle_transition`
//! create dalle migration v3-v6. Nessuna scrittura, nessuna
//! validazione referenziale (garantita dal codice di scrittura).

use crate::domain::transition::{valid_transitions, Trigger};
use crate::domain::{ParseStatusError, ParseTriggerError, Status, Transition};
use crate::repo::{LifecycleRepo, RepoError};
use sqlx::{Row, SqlitePool};
use tokio::runtime::Runtime;

/// SQLite-backed lifecycle repository.
pub struct SqliteRepo {
    pool: SqlitePool,
    rt: Runtime,
}

impl SqliteRepo {
    /// Builds a repository from an existing [`SqlitePool`].
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self {
            pool,
            rt: Runtime::new().expect("tokio runtime should start"),
        }
    }

    /// Returns a clone of the underlying connection pool.
    pub fn pool(&self) -> Result<SqlitePool, RepoError> {
        Ok(self.pool.clone())
    }

    /// Runs an async sqlx future on the repository's tokio runtime.
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.rt.block_on(future)
    }
}

impl LifecycleRepo for SqliteRepo {
    fn states(&self) -> Result<Vec<Status>, RepoError> {
        let pool = self.pool()?;
        let rows = self.block_on(
            sqlx::query(
                "SELECT name, description, is_visible, is_archivable, is_reopenable, sort_order
                 FROM lifecycle_state ORDER BY sort_order",
            )
            .fetch_all(&pool),
        )
        .map_err(|e| RepoError::Query(e.to_string()))?;

        let mut states = Vec::with_capacity(rows.len());
        for row in rows {
            let name: String = row
                .try_get("name")
                .map_err(|e| RepoError::Query(format!("failed to read name: {e}")))?;
            states.push(
                Status::try_from(name.as_str()).map_err(|e| {
                    RepoError::ParseStatus(name.clone(), e)
                })?,
            );
        }
        Ok(states)
    }

    fn allowed_transitions(&self) -> Result<Vec<Transition>, RepoError> {
        let pool = self.pool()?;
        let rows = self.block_on(
            sqlx::query(
                "SELECT from_state, to_state, trigger, description
                 FROM lifecycle_transition ORDER BY from_state, to_state, trigger",
            )
            .fetch_all(&pool),
        )
        .map_err(|e| RepoError::Query(e.to_string()))?;

        let mut transitions = Vec::with_capacity(rows.len());
        for row in rows {
            let from: String = row
                .try_get("from_state")
                .map_err(|e| RepoError::Query(format!("failed to read from_state: {e}")))?;
            let to: String = row
                .try_get("to_state")
                .map_err(|e| RepoError::Query(format!("failed to read to_state: {e}")))?;
            let trigger: String = row
                .try_get("trigger")
                .map_err(|e| RepoError::Query(format!("failed to read trigger: {e}")))?;
            let description: String = row
                .try_get("description")
                .map_err(|e| RepoError::Query(format!("failed to read description: {e}")))?;

            transitions.push(Transition {
                from: Status::try_from(from.as_str()).map_err(|e| {
                    RepoError::ParseStatus(from.clone(), e)
                })?,
                to: Status::try_from(to.as_str()).map_err(|e| {
                    RepoError::ParseStatus(to.clone(), e)
                })?,
                trigger: Trigger::try_from(trigger.as_str()).map_err(|e| {
                    RepoError::ParseTrigger(trigger.clone(), e)
                })?,
                description,
            });
        }
        Ok(transitions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions_has_seven_entries() {
        assert_eq!(valid_transitions().len(), 7);
    }
}
