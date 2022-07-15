#![forbid(unsafe_code)]

pub use async_trait::async_trait;
pub use bytes::{Bytes, BytesMut};
pub use headers;
pub use http::{header, Method, StatusCode};
pub use hyper::Body;
pub use std::future::Future;
pub use thiserror::Error as ThisError;

pub type Result<T, E = Error> = core::result::Result<T, E>;
pub type Request<T = Body> = http::Request<T>;
pub type Response<T = Body> = http::Response<T>;

#[macro_use]
pub(crate) mod macros;

pub mod handler;
#[doc(no_inline)]
pub use crate::handler::{
    BoxHandler, FnExt, Handler, HandlerExt, Next, Responder, ResponderExt, Transform,
};

pub mod middleware;
pub mod types;

mod error;
mod from_request;
mod into_response;
mod request;
mod response;

pub use error::Error;
pub use from_request::FromRequest;
pub use into_response::IntoResponse;
pub use request::RequestExt;
pub use response::ResponseExt;

#[doc(hidden)]
mod tuples {
    use super::*;

    tuple_impls!(A B C D E F G H I J K L);
}
