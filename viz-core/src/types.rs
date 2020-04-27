use std::sync::Arc;

pub use async_trait::async_trait;
pub(crate) use handle::Handle;
pub use std::future::Future;
pub use std::pin::Pin;

pub use futures_core::future::{BoxFuture, LocalBoxFuture};

pub type Error = anyhow::Error;

pub type Result<T = (), E = Error> = anyhow::Result<T, E>;

pub type Middleware = Vec<Arc<dyn for<'a> Handle<'a, crate::Context, Result<crate::Response>>>>;
