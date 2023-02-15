use crate::State;
use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

///
#[derive(Clone, Debug, PartialEq, Eq, GraphQLEnum)]
pub enum ClientMessageKind {
    Err,
    Warning,
    Info,
}

///
#[derive(Clone, Debug, PartialEq, Eq, GraphQLEnum)]
pub enum ClientMessageSource {
    Dashboard,
    Client,
    Mix,
    Statistics,
}

///
#[derive(Clone, Debug, Eq, GraphQLObject, PartialEq)]
pub struct ClientMessage {
    pub kind: ClientMessageKind,
    pub message: String,
    pub source: ClientMessageSource,
}

#[derive(Debug)]
pub struct ConsoleLogger {
    pub state: State,
}

impl ConsoleLogger {
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self { state }
    }

    pub fn log_message(
        &self,
        message: String,
        kind: ClientMessageKind,
        source: ClientMessageSource,
    ) {
        let mut console_log = self.state.console_log.lock_mut();
        console_log.push(ClientMessage {
            message,
            kind,
            source,
        })
    }
}
