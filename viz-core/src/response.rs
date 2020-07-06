use std::{
    any::TypeId,
    borrow::Cow,
    error::Error as StdError,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{http, Error, Result};

pub struct Response {
    pub(crate) raw: http::Response,
}

impl StdError for Response {}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("header", &self.headers())
            .finish()
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("display {}", self.status());
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("header", &self.headers())
            .finish()
    }
}

impl Response {
    pub fn new() -> Self {
        Self {
            raw: http::Response::new(http::Body::empty()),
        }
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
