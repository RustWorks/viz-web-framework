//! Viz's Core

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]

pub use std::future::Future;

use viz_utils::anyhow;

mod context;
mod extract;
mod handler;
mod macros;
mod response;

pub mod config;
pub mod types;

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "sse")]
pub mod sse;
#[cfg(feature = "ws")]
pub mod ws;

#[allow(missing_docs)]
pub mod http {
    pub use headers;
    pub use http::*;
    pub use hyper::Body;
    pub use hyper::Error;
    pub use mime;

    pub type Request<T = Body> = ::http::Request<T>;
    pub type Response<T = Body> = ::http::Response<T>;
}

/// Error
pub use anyhow::{anyhow, bail, ensure, Error};

/// Result
pub type Result<T = Response, E = anyhow::Error> = anyhow::Result<T, E>;

/// Dyn Middleware
pub type DynMiddleware = dyn for<'a> Middleware<'a, Context, Output = Result>;

/// Middleware List
pub type VecMiddleware = Vec<std::sync::Arc<DynMiddleware>>;

/// Middleware Extends
pub trait MiddlewareExt<'a>: Middleware<'a, Context> {}

pub use context::Context;
pub use extract::Extract;
pub use handle::{BoxFuture, Handle as Middleware};
pub use handler::{Endpoint, Handler};
pub use response::Response;

/// Responds a custom error to response.
#[macro_export]
macro_rules! reject {
    ($err:expr) => {
        return Err(how!($err));
    };
}

/// Converts a custom error to [`Response`] and then converts to [`Error`].
#[macro_export]
macro_rules! how {
    ($err:expr) => {
        Into::<Error>::into(Into::<Response>::into($err))
    };
}
