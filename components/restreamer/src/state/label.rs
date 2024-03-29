use derive_more::{Deref, Display, Into};
use juniper::{GraphQLScalar, InputValue, ScalarValue};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;

/// Label of a [`Restream`] or an [`Output`].
///
/// [`Restream`]: crate::state::Restream
/// [`Output`]: crate::state::Output
#[derive(
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    Hash,
    Into,
    PartialEq,
    Serialize,
    GraphQLScalar,
)]
#[graphql(from_input_with = Self::from_input, transparent)]
pub struct Label(String);

const MAX_LABEL_LENGTH: usize = 70;

impl Label {
    /// Creates a new [`Label`] if the given value meets its invariants.
    #[must_use]
    pub fn new<'s, S: Into<Cow<'s, str>>>(val: S) -> Option<Self> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(&format!(r"^[^,\n\t\r\f\v]{{1,{MAX_LABEL_LENGTH}}}$"))
                .unwrap()
        });

        let val = val.into();
        (!val.is_empty() && REGEX.is_match(&val))
            .then(|| Self(val.into_owned()))
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let s = v
            .as_scalar()
            .and_then(ScalarValue::as_str)
            .and_then(Self::new);

        match s {
            None => Err(format!(
                "Some characters are invalid \
                or length is more then {MAX_LABEL_LENGTH} in: {v}"
            )),
            Some(e) => Ok(e),
        }
    }
}

impl<'de> Deserialize<'de> for Label {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(<Cow<'_, str>>::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::custom("Not a valid Label"))
    }
}
