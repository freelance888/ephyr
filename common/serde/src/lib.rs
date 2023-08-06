//! Utils and helpers for serialization/deserialization with [`serde`].

#![deny(
    broken_intra_doc_links,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

#[cfg(feature = "seconds")]
pub mod seconds;
#[cfg(feature = "timelike")]
pub mod timelike;
#[cfg(feature = "timezone")]
pub mod timezone;

/// Indicates whether the given [`bool`] is `false`.
///
/// # Purpose
///
/// Signature of this function matches for a convenient use in a
/// `skip_serializing_if` [`serde`]'s attribute.
#[allow(clippy::trivially_copy_pass_by_ref)] // required for `serde`
#[inline]
#[must_use]
pub fn is_false(val: &bool) -> bool {
    !*val
}
