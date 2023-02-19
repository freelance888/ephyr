//! Logger for messages displaying in UI console

use crate::State;
use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

/// Type of message
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq)]
pub enum ConsoleMessageKind {
    /// Indicates error message
    Err,
    /// Indicates warning message
    Warning,
    /// Indicates informational message
    Info,
}

/// The source of message
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq)]
pub enum ConsoleMessageSource {
    /// Message came from Dashboard app
    Dashboard,
    /// Message came from Client app
    Client,
    /// Message came from statistics subsystem
    Statistics,
}

/// Message for console
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct ConsoleMessage {
    /// Kind of message: Error, Warning, Info
    pub kind: ConsoleMessageKind,
    /// Message itself
    pub message: String,
    /// Source of message, i.e. what subsystem or app sent it
    pub source: ConsoleMessageSource,
}

/// Manages publishing messages into messages list
#[derive(Debug)]
pub struct ConsoleLogger {
    /// Reference to [`State`]
    pub state: State,
}

impl ConsoleLogger {
    /// Creates new instance of [`ConsoleLogger`]
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self { state }
    }

    /// Add message with specific [`ConsoleMessageKind`] from
    /// specific [`ConsoleMessageSource`]
    pub fn log_message(
        &self,
        message: String,
        kind: ConsoleMessageKind,
        source: ConsoleMessageSource,
    ) {
        let mut console_log = self.state.console_log.lock_mut();
        console_log.push(ConsoleMessage {
            kind,
            message,
            source,
        });
    }
}
