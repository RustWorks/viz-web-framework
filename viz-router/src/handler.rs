use std::sync::Arc;

use viz_core::{http, Context, DynMiddleware, Guard, Middleware, Response, Result};
use viz_utils::futures::future::BoxFuture;

pub struct RouteHandler {
    guard: Arc<Box<dyn Guard>>,
    handler: Arc<DynMiddleware>,
}

impl RouteHandler {
    pub fn new(guard: Arc<Box<dyn Guard>>, handler: Arc<DynMiddleware>) -> Self {
        Self { guard, handler }
    }
}

impl<'a> Middleware<'a, Context> for RouteHandler {
    type Output = Result<Response>;

    #[inline]
    fn call(&'a self, cx: &'a mut Context) -> BoxFuture<'a, Self::Output> {
        if self.guard.check(cx) {
            return self.handler.call(cx);
        }

        Box::pin(async { Ok(http::StatusCode::NOT_FOUND.into()) })
    }
}
