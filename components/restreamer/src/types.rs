//! Common types
use fmt::Debug;
use futures::future;
use juniper::{
    GraphQLScalar, InputValue, ParseScalarResult, ParseScalarValue,
    ScalarToken, ScalarValue,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// Abort handle of a future.
#[derive(Clone, Debug)]
pub struct DroppableAbortHandle(future::AbortHandle);

impl DroppableAbortHandle {
    /// Creates and initialise callback for aborting future on `drop()`
    #[must_use]
    pub fn new(callback: future::AbortHandle) -> Self {
        Self(callback)
    }
}

impl Drop for DroppableAbortHandle {
    #[inline]
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Generic number for using with Graphql
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, GraphQLScalar,
)]
pub struct UNumber(u16);

impl UNumber {
    /// Creates new instance of [`UNumber`]
    #[must_use]
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    fn to_output<S: ScalarValue>(&self) -> juniper::Value<S> {
        juniper::Value::scalar(self.0.to_owned().to_string())
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let v = v
            .as_scalar()
            .and_then(ScalarValue::as_int)
            .and_then(|v| u16::try_from(v).ok());
        match v {
            Some(n) => Ok(UNumber::new(n)),
            _ => Err("Error converting UNumber(u16) from i32".to_string()),
        }
    }

    fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}
