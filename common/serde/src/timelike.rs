//! Custom [`serde`] serialization/deserialization functions for [`Duration`]
//! in a `%H:%M:%S` time-like format (`123:05:01`, for example).

use std::{borrow::Cow, time::Duration};

use serde::{de::Error as _, Deserialize as _, Deserializer, Serializer};

/// Formats the given [`Duration`] as a `%H:%M:%S` time-like string (`03:05:01`,
/// for example).
#[must_use]
pub fn format(duration: &Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    format!("{:02}:{:02}:{:02}", mins / 60, mins % 60, secs % 60)
}

/// Serializes [`Duration`] into a `%H:%M:%S` time-like format (`123:05:01`,
/// for example).
///
/// # Errors
///
/// Never errors.
#[inline]
pub fn serialize<S>(dur: &Duration, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&format(dur))
}

/// Deserializes [`Duration`] from a `%H:%M:%S` time-like format (`123:05:01`,
/// for example).
///
/// # Errors
///
/// If an input is not time-like formatted or does contain invalid time.
pub fn deserialize<'a, D>(de: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'a>,
{
    let s = <Cow<'_, str>>::deserialize(de)?;
    let mut iter = s.split(':');

    let hours: u64 = iter
        .next()
        .ok_or_else(|| D::Error::custom("no hours specified"))?
        .parse()
        .map_err(|e| D::Error::custom(format!("cannot parse hours: {e}")))?;

    let mins: u64 = iter
        .next()
        .ok_or_else(|| D::Error::custom("no minutes specified"))?
        .parse()
        .map_err(|e| D::Error::custom(format!("cannot parse minutes: {e}")))?;
    if mins >= 60 {
        return Err(D::Error::custom(format!("invalid minutes: {mins}")));
    }

    let secs: u64 = iter
        .next()
        .ok_or_else(|| D::Error::custom("no seconds specified"))?
        .parse()
        .map_err(|e| D::Error::custom(format!("cannot parse seconds: {e}")))?;
    if secs >= 60 {
        return Err(D::Error::custom(format!("invalid seconds: {secs}")));
    }

    if iter.next().is_some() {
        return Err(D::Error::custom(format!("invalid format: {s}")));
    }

    Ok(Duration::from_secs(hours * 3600 + mins * 60 + secs))
}

/// [`Option`] support.
pub mod opt {
    use std::time::Duration;

    use serde::{de::Deserializer, ser::Serializer, Deserialize};

    /// Serializes [`Option`]ed [`Duration`] into a `%H:%M:%S` time-like format.
    ///
    /// # Errors
    ///
    /// Never errors.
    #[inline]
    pub fn serialize<S>(
        dur: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dur.as_ref() {
            Some(d) => super::serialize(d, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes [`Option`]ed [`Duration`] from a `%H:%M:%S` time-like
    /// format.
    ///
    /// # Errors
    ///
    /// If an input is not time-like formatted or does contain invalid time.
    pub fn deserialize<'a, D>(d: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'a>,
    {
        #[derive(Deserialize)]
        struct OptDuration(
            #[serde(deserialize_with = "super::deserialize")] Duration,
        );

        Ok(Option::<OptDuration>::deserialize(d)?.map(|d| d.0))
    }
}

#[cfg(test)]
mod spec {
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct Test(#[serde(with = "super")] Duration);

    #[test]
    fn serializes_correctly() {
        for (input, expected) in &[
            (0, r#""00:00:00""#),
            (1, r#""00:00:01""#),
            (60, r#""00:01:00""#),
            (3600, r#""01:00:00""#),
            (86400, r#""24:00:00""#),
            (10 * 86400 + 67, r#""240:01:07""#),
            (324, r#""00:05:24""#),
        ] {
            let input = Test(Duration::from_secs(*input));
            let actual =
                serde_json::to_string(&input).expect("Failed to serialize");

            assert_eq!(actual, expected.to_string());
        }
    }

    #[test]
    fn deserializes_correctly() {
        for (input, expected) in &[
            (r#""00:00:00""#, 0),
            (r#""00:00:01""#, 1),
            (r#""00:01:00""#, 60),
            (r#""01:00:00""#, 3600),
            (r#""24:00:00""#, 86400),
            (r#""240:01:07""#, 10 * 86400 + 67),
            (r#""00:05:24""#, 324),
        ] {
            let actual: Test =
                serde_json::from_str(*input).expect("Failed to deserialize");

            assert_eq!(actual.0, Duration::from_secs(*expected));
        }
    }
}
