//! Logger for messages displaying in UI console

use crate::State;
use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

/// Type of message
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq)]
pub enum ConsoleMessageKind {
    Err,
    Warning,
    Info,
}

/// Source of message
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq)]
pub enum ConsoleMessageSource {
    Dashboard,
    Client,
    Mix,
    Statistics,
}

/// Message for console
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct ConsoleMessage {
    pub kind: ConsoleMessageKind,
    pub message: String,
    pub source: ConsoleMessageSource,
}

/// Add errors, messages, warnings to client's console
#[derive(Debug)]
pub struct ConsoleLogger {
    /// Reference to [`State`]
    pub state: State,
}

impl ConsoleLogger {
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self { state }
    }

    /// Add message to console
    pub fn log_message(
        &self,
        message: String,
        kind: ConsoleMessageKind,
        source: ConsoleMessageSource,
    ) {
        let mut console_log = self.state.console_log.lock_mut();
        console_log.push(ConsoleMessage {
            message,
            kind,
            source,
        })
    }
}
