//! Delay of a [`Mixin`] being mixed with an [`Output`].
use juniper::{
    GraphQLScalar, InputValue, ParseScalarResult, ParseScalarValue,
    ScalarToken, ScalarValue, Value,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, time::Duration};

/// Delay of a [`Mixin`] being mixed with an [`Output`].
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    GraphQLScalar,
)]
#[graphql(with = Self)]
pub struct Delay(#[serde(with = "serde_humantime")] Duration);

impl Delay {
    /// Creates a new [`Delay`] out of the given milliseconds.
    #[inline]
    #[must_use]
    pub fn from_millis<N: TryInto<u64>>(millis: N) -> Option<Self> {
        millis
            .try_into()
            .ok()
            .map(|m| Self(Duration::from_millis(m)))
    }

    /// Returns milliseconds of this [`Delay`].
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn as_millis(&self) -> i32 {
        self.0.as_millis().try_into().unwrap()
    }

    /// Indicates whether this [`Delay`] introduces no actual delay.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == Duration::default()
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(self.as_millis())
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let v = v
            .as_scalar()
            .and_then(ScalarValue::as_int)
            .and_then(Self::from_millis);
        match v {
            None => Err("test".to_string()),
            Some(d) => Ok(d),
        }
    }

    fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}
