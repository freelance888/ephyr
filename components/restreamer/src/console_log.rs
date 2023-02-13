use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

///
#[derive(Clone, Debug, PartialEq, Eq, GraphQLEnum)]
pub enum ClientMessageKind {
    Error,
    Warning,
    Info,
}

///
#[derive(Clone, Debug, Eq, GraphQLObject, PartialEq)]
pub struct ClientMessage {
    pub kind: ClientMessageKind,
    pub message: String,
    pub source: String,
}
