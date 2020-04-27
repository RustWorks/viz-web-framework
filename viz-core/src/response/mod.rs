use crate::Error;
use crate::Result;

#[derive(Debug)]
pub struct Response {}

impl Response {
    pub fn new() -> Self {
        Self {}
    }
}

impl From<Error> for Response {
    fn from(_e: Error) -> Self {
        // @TODO: convert error to response
        Self {}
    }
}

impl<T, E> From<Result<T, E>> for Response
where
    T: Into<Response>,
    E: Into<Response>,
{
    fn from(r: Result<T, E>) -> Self {
        r.map_or_else(Into::into, Into::into)
    }
}

impl From<String> for Response {
    fn from(_s: String) -> Self {
        Self {}
    }
}

impl<'a> From<&'a str> for Response {
    fn from(_s: &'a str) -> Self {
        Self {}
    }
}

impl<'a> From<&'a [u8]> for Response {
    fn from(_s: &'a [u8]) -> Self {
        Self {}
    }
}
