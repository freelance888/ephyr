//! Volume rate of an audio track in percents and flag if it is muted.
use crate::spec;
use juniper::{
    GraphQLObject, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, ScalarToken, ScalarValue, Value,
};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::convert::{TryFrom, TryInto};

/// Volume rate of an audio track in percents and flag if it is muted.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct Volume {
    /// Volume rate or level
    pub level: VolumeLevel,
    /// Whether it is muted or not
    pub muted: bool,
}

impl Volume {
    /// Value of a [`Volume`] rate corresponding to the original one of an audio
    /// track.
    pub const ORIGIN: Volume = Volume {
        level: VolumeLevel::ORIGIN,
        muted: false,
    };

    /// Creates a new [`Volume`] rate value if it satisfies the required
    /// invariants:
    /// - within [`VolumeLevel::OFF`] and [`VolumeLevel::MAX`] values.
    #[must_use]
    pub fn new(num: &spec::v1::Volume) -> Self {
        VolumeLevel::new(num.level.0).map_or_else(Self::default, |volume| {
            Self {
                level: volume,
                muted: num.muted,
            }
        })
    }

    /// Displays this [`Volume`] as a fraction of `1`, i.e. `100%` as `1`, `50%`
    /// as `0.50`, and so on.
    #[must_use]
    pub fn display_as_fraction(self) -> String {
        if self.muted {
            String::from("0.00")
        } else {
            format!("{}.{:02}", self.level.0 / 100, self.level.0 % 100)
        }
    }

    /// Indicates whether this [`Volume`] rate value corresponds is the
    /// [`Volume::ORIGIN`]al one.
    #[allow(clippy::trivially_copy_pass_by_ref)] // required for `serde`
    #[inline]
    #[must_use]
    pub fn is_origin(&self) -> bool {
        *self == Self::ORIGIN
    }

    /// Export this struct as [`spec::v1::Volume`]
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::Volume {
        spec::v1::Volume {
            level: self.level,
            muted: self.muted,
        }
    }
}

/// Default value for Volume is [`Volume::ORIGIN`]
impl Default for Volume {
    fn default() -> Self {
        Volume::ORIGIN
    }
}

/// For backward compatibility can convert from number to Volume struct
/// the `#[serde(try_from='VolumeLevel')]` in [Volume] must be enabled
impl TryFrom<VolumeLevel> for Volume {
    type Error = std::num::ParseIntError;
    fn try_from(value: VolumeLevel) -> Result<Self, Self::Error> {
        Ok(Volume {
            level: value,
            muted: false,
        })
    }
}

/// Volume rate of an audio track in percents.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    SmartDefault,
    GraphQLScalar,
)]
#[graphql(with = Self)]
pub struct VolumeLevel(#[default(Volume::ORIGIN.level.0)] u16);
impl VolumeLevel {
    /// Maximum possible value of a [`VolumeLevel`].
    pub const MAX: VolumeLevel = VolumeLevel(1000);

    /// Value of a [`Volume`] rate corresponding to the original one of an audio
    /// track.
    pub const ORIGIN: VolumeLevel = VolumeLevel(100);

    /// Minimum possible value of a [`Volume`] rate. Actually, disables audio.
    pub const OFF: VolumeLevel = VolumeLevel(0);
    /// Creates a new [`VolumeLevel`] rate value if it satisfies the required
    /// invariants:
    /// - within [`VolumeLevel::OFF.level`] and [`VolumeLevel::MAX.level`]
    /// values.
    pub fn new<N: TryInto<u16>>(val: N) -> Option<Self> {
        let num = val.try_into().ok()?;
        if (VolumeLevel::OFF.0..=VolumeLevel::MAX.0).contains(&num) {
            Some(Self(num))
        } else {
            None
        }
    }

    #[allow(clippy::wrong_self_convention, clippy::trivially_copy_pass_by_ref)]
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(i32::from(self.0))
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let s = v
            .as_scalar()
            .and_then(ScalarValue::as_int)
            .and_then(Self::new);
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
            .or_else(|_| <i32 as ParseScalarValue<S>>::from_str(value))
    }
}

/// Type of a `Mixin` delay in milliseconds.
///
/// Negative values are not allowed.
#[cfg(test)]
mod volume_spec {
    use super::{Volume, VolumeLevel};
    use crate::spec::v1;

    #[test]
    fn displays_as_fraction() {
        for (input, expected) in &[
            (
                v1::Volume {
                    level: VolumeLevel(1),
                    muted: false,
                },
                "0.01",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(10),
                    muted: false,
                },
                "0.10",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(200),
                    muted: false,
                },
                "2.00",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(107),
                    muted: false,
                },
                "1.07",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(170),
                    muted: false,
                },
                "1.70",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(1000),
                    muted: false,
                },
                "10.00",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(0),
                    muted: false,
                },
                "0.00",
            ),
            (
                v1::Volume {
                    level: VolumeLevel(200),
                    muted: true,
                },
                "0.00",
            ),
        ] {
            let actual = Volume::new(input).display_as_fraction();
            assert_eq!(&actual, *expected);
        }
    }
}
