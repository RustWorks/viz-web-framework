use std::marker::PhantomData;

use crate::{BoxFuture, Context, Extract, Future, Middleware, Response, Result};

/// The handle trait within the given Args then returns Response.
pub trait Handler<Args>: Clone + 'static {
    /// The type of value produced on completion.
    type Output: Into<Response>;
    /// Retures a future for handler.
    type Future: Future<Output = Self::Output> + Send + 'static;

    /// Invokes the handler within the given args.
    fn call(&self, args: Args) -> Self::Future;
}

/// Endpoint
#[derive(Debug, Clone)]
pub struct Endpoint<Handler, Args>(Handler, PhantomData<fn(Args)>);

impl<Handler, Args> Endpoint<Handler, Args> {
    /// Creates new Endpoint with a [Handler]
    pub fn new(h: Handler) -> Self {
        Self(h, PhantomData)
    }
}

impl<'a, H, A> Middleware<'a, Context> for Endpoint<H, A>
where
    A: Extract + Send + 'static,
    A::Error: Into<Response> + Send,
    H: Handler<A> + Send + Sync + 'static,
    H::Output: Into<Response>,
    H::Future: Future<Output = H::Output> + Send + 'static,
{
    type Output = Result;

    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        Box::pin(async move {
            Ok(match A::extract(cx).await {
                Ok(args) => Handler::call(&self.0, args).await.into(),
                Err(e) => e.into(),
            })
        })
    }
}
