//! Custom [`serde`] serialization/deserialization functions for [`TimeZone`]
//! in a [RFC 3339 format][1] (`+04:03`, for example).
//!
//! [`TimeZone`]: chrono::FixedOffset
//! [1]: https://tools.ietf.org/html/rfc3339#section-4.2

use std::{borrow::Cow, convert::TryFrom as _};

use chrono::FixedOffset as TimeZone;
use serde::{de::Error as _, Deserialize as _, Deserializer, Serializer};

/// Serializes [`TimeZone`] in a [RFC 3339 format][1] (`+04:03`, for example).
///
/// # Errors
///
/// Never errors.
///
/// [1]: https://tools.ietf.org/html/rfc3339#section-4.2
#[allow(clippy::trivially_copy_pass_by_ref)]
#[inline]
pub fn serialize<S>(tz: &TimeZone, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&format!("{tz:?}"))
}

/// Deserializes [`TimeZone`] from a [RFC 3339 format][1] (`+04:03`, for
/// example).
///
/// # Errors
///
/// If an input is not [RFC 3339 formatted][1] timezone or does contain invalid
/// timezone.
///
/// [1]: https://tools.ietf.org/html/rfc3339#section-4.2
pub fn deserialize<'a, D>(de: D) -> Result<TimeZone, D::Error>
where
    D: Deserializer<'a>,
{
    let s = <Cow<'_, str>>::deserialize(de)?;
    let (sign, s) = match s.chars().next() {
        Some('+') => (1, &s[1..]),
        Some('-') => (-1, &s[1..]),
        Some(_) => (1, &*s),
        None => return Err(D::Error::custom(format!("invalid timezone: {s}"))),
    };
    let mut iter = s.split(':');

    let hours: u32 = iter
        .next()
        .ok_or_else(|| D::Error::custom("no hours specified"))?
        .parse()
        .map_err(|e| D::Error::custom(format!("cannot parse hours: {e}")))?;

    let mins: u32 = iter
        .next()
        .ok_or_else(|| D::Error::custom("no minutes specified"))?
        .parse()
        .map_err(|e| D::Error::custom(format!("cannot parse minutes: {e}")))?;
    if mins >= 60 {
        return Err(D::Error::custom(format!("invalid minutes: {mins}")));
    }

    let secs: u32 = if let Some(s) = iter.next() {
        s.parse().map_err(|e| {
            D::Error::custom(format!("cannot parse seconds: {e}"))
        })?
    } else {
        0
    };
    if secs >= 60 {
        return Err(D::Error::custom(format!("invalid seconds: {secs}")));
    }

    #[allow(clippy::map_err_ignore)]
    let total_secs = i32::try_from(hours * 3600 + mins * 60 + secs)
        .map_err(|_| D::Error::custom(format!("invalid timezone: {s}")))?;

    TimeZone::east_opt(sign * total_secs)
        .ok_or_else(|| D::Error::custom(format!("invalid timezone: {s}")))
}

/// [`Option`] support.
pub mod opt {
    use chrono::FixedOffset as TimeZone;
    use serde::{de::Deserializer, ser::Serializer, Deserialize};

    /// Serializes [`Option`]ed [`TimeZone`] in a [RFC 3339 format][1]
    /// (`+04:03`, for example).
    ///
    /// # Errors
    ///
    /// Never errors.
    ///
    /// [1]: https://tools.ietf.org/html/rfc3339#section-4.2
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[inline]
    pub fn serialize<S>(
        tz: &Option<TimeZone>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match tz.as_ref() {
            Some(tz) => super::serialize(tz, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes [`Option`]ed [`TimeZone`] from a [RFC 3339 format][1]
    /// (`+04:03`, for example).
    ///
    /// # Errors
    ///
    /// If an input is not [RFC 3339 formatted][1] timezone or does contain
    /// invalid timezone.
    ///
    /// [1]: https://tools.ietf.org/html/rfc3339#section-4.2
    pub fn deserialize<'a, D>(d: D) -> Result<Option<TimeZone>, D::Error>
    where
        D: Deserializer<'a>,
    {
        #[derive(Deserialize)]
        struct OptTimeZone(
            #[serde(deserialize_with = "super::deserialize")] TimeZone,
        );

        Ok(Option::<OptTimeZone>::deserialize(d)?.map(|d| d.0))
    }
}

#[cfg(test)]
mod spec {
    use chrono::FixedOffset as TimeZone;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct Test(#[serde(with = "super")] TimeZone);

    #[test]
    fn serializes_correctly() {
        let (sec, min, hour) = (1, 60, 3600);
        for (input, expected) in &[
            (0, r#""+00:00""#),
            (sec, r#""+00:00:01""#),
            (-sec, r#""-00:00:01""#),
            (min, r#""+00:01""#),
            (-min, r#""-00:01""#),
            (hour, r#""+01:00""#),
            (-hour, r#""-01:00""#),
            (5 * hour + 30 * min, r#""+05:30""#),
            (-(5 * hour + 30 * min + 15 * sec), r#""-05:30:15""#),
        ] {
            let input = Test(TimeZone::east(*input));
            let actual =
                serde_json::to_string(&input).expect("Failed to serialize");

            assert_eq!(actual, expected.to_string());
        }
    }

    #[test]
    fn deserializes_correctly() {
        let (sec, min, hour) = (1, 60, 3600);
        for (input, expected) in &[
            (r#""+00:00""#, 0),
            (r#""00:00""#, 0),
            (r#""00:00:00""#, 0),
            (r#""-00:00""#, 0),
            (r#""+00:00:01""#, sec),
            (r#""-00:00:01""#, -sec),
            (r#""+00:01""#, min),
            (r#""+00:01:00""#, min),
            (r#""-00:01""#, -min),
            (r#""+01:00""#, hour),
            (r#""-01:00""#, -hour),
            (r#""-01:00:00""#, -hour),
            (r#""+05:30""#, 5 * hour + 30 * min),
            (r#""-05:30:15""#, -(5 * hour + 30 * min + 15 * sec)),
        ] {
            let actual: Test =
                serde_json::from_str(*input).expect("Failed to deserialize");

            assert_eq!(actual.0, TimeZone::east(*expected));
        }
    }
}
