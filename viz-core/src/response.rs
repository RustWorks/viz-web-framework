use std::{
    any::TypeId,
    borrow::Cow,
    error::Error as StdError,
    fmt,
    ops::{Deref, DerefMut},
};

use viz_utils::serde;

use crate::{http, Error, Result};

/// Viz Response
pub struct Response {
    pub(crate) raw: http::Response,
}

impl StdError for Response {}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("header", &self.headers())
            .finish()
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("header", &self.headers())
            .finish()
    }
}

impl Response {
    /// Creates a response
    pub fn new() -> Self {
        Self {
            raw: http::Response::new(http::Body::empty()),
        }
    }

    /// Responds Text
    pub fn text(data: impl Into<http::Body>) -> Self {
        let mut raw = http::Response::new(data.into());
        raw.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static(mime::TEXT_PLAIN.as_ref()),
        );
        Self { raw }
    }

    /// Responds HTML
    pub fn html(data: impl Into<http::Body>) -> Self {
        let mut raw = http::Response::new(data.into());
        raw.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
        );
        Self { raw }
    }

    /// Responds JSON
    pub fn json(data: impl Into<http::Body>) -> Self {
        let mut raw = http::Response::new(data.into());
        raw.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        Self { raw }
    }

    /// Sets status for response
    pub fn with_status(mut self, status: http::StatusCode) -> Self {
        *self.status_mut() = status;
        self
    }
}

impl Deref for Response {
    type Target = http::Response;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl Into<http::Response> for Response {
    fn into(self) -> http::Response {
        self.raw
    }
}

impl From<http::Response> for Response {
    fn from(raw: http::Response) -> Self {
        Self { raw }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        let mut raw = http::Response::new(http::Body::from(e.to_string()));
        *raw.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
        Self { raw }
    }
}

impl<T, E> From<Result<T, E>> for Response
where
    T: Into<Response>,
    E: Into<Response> + Into<Error> + 'static,
{
    fn from(r: Result<T, E>) -> Self {
        r.map_or_else(
            |e| {
                if TypeId::of::<Error>() == TypeId::of::<E>() {
                    Into::<Error>::into(e)
                        .downcast::<Response>()
                        .map_or_else(Into::into, Into::into)
                } else {
                    e.into()
                }
            },
            Into::into,
        )
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        Self {
            raw: http::Response::new(http::Body::from(s)),
        }
    }
}

impl From<&'_ str> for Response {
    fn from(s: &'_ str) -> Self {
        Self {
            raw: http::Response::new(s.to_owned().into()),
        }
    }
}

impl From<Cow<'_, str>> for Response {
    fn from(s: Cow<'_, str>) -> Self {
        s.into()
    }
}

impl From<&'_ [u8]> for Response {
    fn from(s: &'_ [u8]) -> Self {
        Self {
            raw: http::Response::new(http::Body::from(s.to_owned())),
        }
    }
}

impl From<()> for Response {
    fn from(_: ()) -> Self {
        Self {
            raw: http::Response::new(http::Body::empty()),
        }
    }
}

impl From<http::StatusCode> for Response {
    fn from(s: http::StatusCode) -> Self {
        let mut res = Response::new();
        *res.status_mut() = s;
        // *res.body_mut() = s.to_string().into();
        res
    }
}

impl<T> From<(http::StatusCode, T)> for Response
where
    T: Into<Response>,
{
    fn from(t: (http::StatusCode, T)) -> Self {
        let mut res = t.1.into();
        *res.status_mut() = t.0;
        res
    }
}

impl From<serde::json::Value> for Response {
    fn from(v: serde::json::Value) -> Self {
        match serde::json::to_vec(&v) {
            Ok(d) => Self::json(d),
            Err(e) => Into::<Error>::into(e).into(),
        }
    }
}
