use crate::state::ClientStatisticsResponse;
use derive_more::{Deref, Display, Into};
use juniper::{
    GraphQLObject, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, ScalarToken, ScalarValue, Value,
};

use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/// Client represents server with running `ephyr` app and can return some
/// statistics about status of [`Input`]s, [`Output`]s .
#[derive(Clone, Debug, GraphQLObject, PartialEq, Serialize, Deserialize)]
pub struct Client {
    /// Unique id of client. Url of the host.
    pub id: ClientId,

    /// Statistics for this [`Client`].
    #[serde(skip)]
    pub statistics: Option<ClientStatisticsResponse>,
}

impl Client {
    /// Creates a new [`Client`] passing host or ip address as identity.
    #[must_use]
    pub fn new(client_id: &ClientId) -> Self {
        Self {
            id: client_id.clone(),
            statistics: None,
        }
    }
}

/// ID of a [`Client`].
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
#[graphql(with = Self)]
pub struct ClientId(Url);

impl ClientId {
    /// Constructs [`ClientId`] from string.
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self(url)
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(self.0.as_str().to_owned())
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let s = v
            .as_scalar()
            .and_then(ScalarValue::as_str)
            .and_then(|s| Url::parse(s).ok())
            .map(Self::new);
        match s {
            None => Err(format!("Expected `String` or `Int`, found: {v}")),
            Some(e) => Ok(e),
        }
    }

    fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl<'de> Deserialize<'de> for ClientId {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(Url::deserialize(deserializer)?))
    }
}
