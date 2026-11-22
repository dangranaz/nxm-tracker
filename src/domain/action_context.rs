//! `ActionContext` — attribuzione obbligatoria per le scritture del lifecycle.
//!
//! Definisce il contesto di azione con i campi obbligatori `user` e
//! `ai_tool`, più `ai_model` opzionale e `auth_method`. Un
//! `ActionContext` senza `user` o `ai_tool` non è costruibile: il
//! costruttore valida che non siano vuoti e restituisce un `Result`
//! tipizzato. Nessun I/O, nessun database, nessun side-effect.

use std::fmt;

use thiserror::Error;

/// Metodo di autenticazione dell'azione. Al momento solo `Local`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthMethod {
    Local,
}

impl fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthMethod::Local => f.write_str("local"),
        }
    }
}

/// Errore tipizzato per il parsing di `AuthMethod`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown auth method: {0}")]
pub struct ParseAuthMethodError(pub String);

impl AuthMethod {
    /// Parsa una stringa in un `AuthMethod`. Accetta solo "local".
    pub fn parse(s: &str) -> Result<AuthMethod, ParseAuthMethodError> {
        match s {
            "local" => Ok(AuthMethod::Local),
            other => Err(ParseAuthMethodError(other.to_string())),
        }
    }
}

/// Contesto di azione per le scritture del lifecycle.
///
/// `user` e `ai_tool` sono obbligatori: il costruttore rifiuta valori vuoti
/// (o solo spazi) con un [`ActionContextError`]. `ai_model` è opzionale.
/// `auth_method` default a `Local`.
///
/// # Example
///
/// ```
/// use nxm_tracker::domain::action_context::{ActionContext, AuthMethod};
///
/// let ctx = ActionContext::new(
///     "alice".to_string(),
///     "claude".to_string(),
///     Some("claude-3".to_string()),
///     AuthMethod::Local,
/// )
/// .expect("valid context");
/// assert_eq!(ctx.user, "alice");
/// assert_eq!(ctx.ai_tool, "claude");
/// assert!(ActionContext::new("".to_string(), "claude".to_string(), None, AuthMethod::Local).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContext {
    /// Utente che richiede l'azione (obbligatorio, non vuoto).
    pub user: String,
    /// Strumento AI che esegue l'azione (obbligatorio, non vuoto).
    pub ai_tool: String,
    /// Modello AI usato (opzionale).
    pub ai_model: Option<String>,
    /// Metodo di autenticazione (solo `Local` per ora).
    pub auth_method: AuthMethod,
}

/// Errore tipizzato per la costruzione di `ActionContext`.
#[derive(Debug, Error)]
pub enum ActionContextError {
    /// Il campo `user` è vuoto o solo spazi.
    #[error("action context user must not be empty")]
    EmptyUser,
    /// Il campo `ai_tool` è vuoto o solo spazi.
    #[error("action context ai_tool must not be empty")]
    EmptyAiTool,
}

impl ActionContext {
    /// Costruisce un `ActionContext`, validando che `user` e `ai_tool`
    /// non siano vuoti (o solo spazi).
    ///
    /// # Errors
    /// Restituisce `ActionContextError::EmptyUser` se `user` è vuoto, o
    /// `ActionContextError::EmptyAiTool` se `ai_tool` è vuoto.
    pub fn new(
        user: String,
        ai_tool: String,
        ai_model: Option<String>,
        auth_method: AuthMethod,
    ) -> Result<Self, ActionContextError> {
        if user.trim().is_empty() {
            return Err(ActionContextError::EmptyUser);
        }
        if ai_tool.trim().is_empty() {
            return Err(ActionContextError::EmptyAiTool);
        }
        Ok(Self {
            user,
            ai_tool,
            ai_model,
            auth_method,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_context_valid() {
        let ctx = ActionContext::new(
            "alice".to_string(),
            "claude".to_string(),
            Some("claude-3".to_string()),
            AuthMethod::Local,
        )
        .expect("valid context");
        assert_eq!(ctx.user, "alice");
        assert_eq!(ctx.ai_tool, "claude");
        assert_eq!(ctx.ai_model, Some("claude-3".to_string()));
        assert_eq!(ctx.auth_method, AuthMethod::Local);
    }

    #[test]
    fn action_context_empty_user_rejected() {
        let err = ActionContext::new(
            "   ".to_string(),
            "claude".to_string(),
            None,
            AuthMethod::Local,
        )
        .unwrap_err();
        assert!(matches!(err, ActionContextError::EmptyUser));
    }

    #[test]
    fn action_context_empty_ai_tool_rejected() {
        let err = ActionContext::new(
            "alice".to_string(),
            "".to_string(),
            None,
            AuthMethod::Local,
        )
        .unwrap_err();
        assert!(matches!(err, ActionContextError::EmptyAiTool));
    }

    #[test]
    fn auth_method_parse() {
        assert_eq!(AuthMethod::parse("local").unwrap(), AuthMethod::Local);
        assert!(AuthMethod::parse("token").is_err());
    }
}