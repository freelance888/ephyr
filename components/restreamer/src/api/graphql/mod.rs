//! [GraphQL] APIs provided by application.
//!
//! [GraphQL]: https://graphql.com

pub mod client;
pub mod mix;

use std::{borrow::Cow, convert::Infallible, fmt, ops::Deref};

use actix_web::{http, HttpRequest};
use derive_more::{Display, Error};
use juniper::{
    graphql_value, http::GraphQLResponse, FieldError, IntoFieldError,
    ScalarValue,
};
use send_wrapper::SendWrapper;
use smart_default::SmartDefault;

/// Context containing [`HttpRequest`] for providing additional information when
/// executing GraphQL operations.
#[derive(Clone, Debug)]
pub struct Context(Option<SendWrapper<HttpRequest>>);

impl Context {
    /// Creates new [`Context`] wrapping the given [`HttpRequest`].
    #[inline]
    #[must_use]
    pub fn new(req: HttpRequest) -> Self {
        Self(Some(SendWrapper::new(req)))
    }

    /// Creates a fake [`Context`], which panics on use.
    ///
    /// Intended for situations where we cannot provide [`HttpRequest`] for
    /// operation execution (running introspection locally, for example).
    #[inline]
    #[must_use]
    pub fn fake() -> Self {
        Self(None)
    }

    /// Returns [`cli::Opts`] parameters stored in [`HttpRequest`]'s context.
    ///
    /// [`cli::Opts`]: crate::cli::Opts
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn config(&self) -> &crate::cli::Opts {
        self.app_data::<crate::cli::Opts>().unwrap()
    }

    /// Returns current [`State`] stored in [`HttpRequest`]'s context.
    ///
    /// [`State`]: crate::State
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn state(&self) -> &crate::State {
        self.app_data::<crate::State>().unwrap()
    }
}

impl Deref for Context {
    type Target = HttpRequest;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0.as_ref().unwrap()
    }
}

/// Error returned to the client by GraphQL API.
#[derive(Clone, Debug, Display, Error, SmartDefault)]
#[display(fmt = "{}", message)]
pub struct Error {
    /// Unique literal code of this [`Error`](struct@Error).
    #[default = "UNKNOWN"]
    pub code: Cow<'static, str>,

    /// HTTP status code of this [`Error`](struct@Error).
    #[default(http::StatusCode::INTERNAL_SERVER_ERROR)]
    pub status: http::StatusCode,

    /// Message of this [`Error`](struct@Error).
    #[default = "Unknown error has happened."]
    pub message: Cow<'static, str>,

    /// Backtrace of this [`Error`](struct@Error).
    #[error(not(backtrace))]
    pub backtrace: Option<Vec<String>>,
}

impl Error {
    /// Creates new default [`Error`](struct@Error) with a given unique literal
    /// code applied.
    ///
    /// Code is usually upper-cased, like `USER_NOT_FOUND`.
    ///
    /// Goes as `errors.extensions.code` field of GraphQL response.
    #[inline]
    #[must_use]
    pub fn new<C: Into<Cow<'static, str>>>(code: C) -> Self {
        Self {
            code: code.into(),
            ..Self::default()
        }
    }

    /// Attaches given [`http::StatusCode`] to this [`Error`](struct@Error).
    ///
    /// Goes as `errors.extensions.status` field of GraphQL response.
    #[inline]
    #[must_use]
    pub fn status<S: Into<http::StatusCode>>(mut self, s: S) -> Self {
        self.set_status(s);
        self
    }

    /// Attaches given message to this [`Error`](struct@Error) as required by
    /// [GraphQL errors spec][1].
    ///
    /// Goes as `errors.message` field of GraphQL response.
    ///
    /// [1]: https://facebook.github.io/graphql/June2018/#sec-Errors
    #[inline]
    #[must_use]
    pub fn message<M: fmt::Display + ?Sized>(mut self, m: &M) -> Self {
        self.set_message(m);
        self
    }

    /// Attaches given backtrace to this [`Error`](struct@Error).
    ///
    /// If set, goes as `errors.extensions.backtrace` field of GraphQL response.
    #[inline]
    #[must_use]
    pub fn backtrace<B: fmt::Display + ?Sized>(mut self, bt: &B) -> Self {
        self.set_backtrace(bt);
        self
    }

    /// Sets [`http::StatusCode`] for this [`Error`](struct@Error).
    ///
    /// Goes as `errors.extensions.status` field of GraphQL response.
    #[inline]
    pub fn set_status<S: Into<http::StatusCode>>(&mut self, s: S) {
        self.status = s.into();
    }

    /// Sets given [`Error`](struct@Error)'s message as required by
    /// [GraphQL errors spec][1].
    ///
    /// Goes as `errors.message` field of GraphQL response.
    ///
    /// [1]: https://facebook.github.io/graphql/June2018/#sec-Errors
    #[inline]
    pub fn set_message<M: fmt::Display + ?Sized>(&mut self, m: &M) {
        self.message = format!("{}", m).into();
    }

    /// Sets backtrace of this [`Error`](struct@Error).
    ///
    /// If set, goes as `errors.extensions.backtrace` field of GraphQL response.
    #[inline]
    pub fn set_backtrace<B: fmt::Display + ?Sized>(&mut self, bt: &B) {
        self.backtrace =
            Some(format!("{}", bt).split('\n').map(String::from).collect());
    }
}

impl<S: ScalarValue> IntoFieldError<S> for Error {
    fn into_field_error(self) -> FieldError<S> {
        let size = if self.backtrace.is_some() { 3 } else { 2 };
        let mut extensions = juniper::Object::with_capacity(size);
        let _added_code = extensions
            .add_field("code", graphql_value!(self.code.into_owned()));
        let _added_status = extensions.add_field(
            "status",
            graphql_value!(i32::from(self.status.as_u16())),
        );
        if let Some(backtrace) = self.backtrace {
            let _added_backtrace = extensions.add_field(
                "backtrace",
                juniper::Value::List(
                    backtrace.into_iter().map(juniper::Value::from).collect(),
                ),
            );
        }
        FieldError::new(self.message, graphql_value!(extensions))
    }
}

impl From<Error> for GraphQLResponse<'_> {
    #[inline]
    fn from(err: Error) -> Self {
        Self::error(err.into_field_error())
    }
}

impl From<Infallible> for Error {
    #[inline]
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl From<anyhow::Error> for Error {
    #[inline]
    fn from(err: anyhow::Error) -> Self {
        Self::new("INTERNAL_SERVER_ERROR")
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .message(&err)
    }
}

impl From<serde_json::Error> for Error {
    #[inline]
    fn from(err: serde_json::Error) -> Self {
        Self::new("INVALID_SPEC_JSON")
            .status(http::StatusCode::BAD_REQUEST)
            .message(&err)
    }
}
