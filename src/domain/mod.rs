//! `domain` module — shared domain types and logic.

pub mod action_context;
pub mod transition;

pub use action_context::{ActionContext, ActionContextError, AuthMethod, ParseAuthMethodError};
pub use transition::{is_transition_allowed, Trigger, Transition, valid_transitions, ParseTriggerError};